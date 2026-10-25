use std::collections::{HashMap, HashSet};

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use super::access::editable_project;
use super::file_rules::MAX_FILE_BYTES;
use super::links::{require_link, GitLink, LinkView};
use super::local_files::{self, content_hash, normalize_content, LocalFile};
use super::locks::lock_project;
use super::policy::load_policy;
use super::remote::{self, ChangeKind, RemoteEntry, RemoteRepo};
use super::snapshot::{self, Snapshot, SnapshotEntry, SnapshotUpdate};
use super::sync_plan::remote_changes;
use super::ApiError;
use crate::accounts::signed_in::SignedInUser;
use crate::db::timestamp_now;
use crate::desktop::devices::{notify_devices, DeviceEvent};
use crate::models::Project;
use crate::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    #[default]
    Ask,
    KeepRepository,
    KeepMine,
}

#[derive(Serialize, Default)]
pub struct PullOutcome {
    pub updated: Vec<String>,
    pub removed: Vec<String>,
    pub conflicts: Vec<String>,
    pub too_large: Vec<String>,
    pub applied: bool,
}

#[derive(Deserialize)]
pub struct PullRequest {
    pub resolution: Option<Resolution>,
}

#[derive(Serialize)]
pub struct PullResponse {
    pub link: LinkView,
    #[serde(flatten)]
    pub outcome: PullOutcome,
}

pub struct PullInput<'a> {
    pub project: &'a Project,
    pub link: &'a GitLink,
    pub head: &'a str,
    pub entries: Vec<RemoteEntry>,
    pub local: HashMap<String, LocalFile>,
    pub snapshot: Snapshot,
    pub resolution: Resolution,
    pub remove_local_only: bool,
    pub user_id: &'a str,
}

struct Download {
    path: String,
    sha: String,
    content: Vec<u8>,
    hash: String,
}

enum Action {
    Write(String, Vec<u8>),
    Remove(String),
}

fn is_locally_changed(path: &str, local: &HashMap<String, LocalFile>, snapshot: &Snapshot) -> bool {
    match (local.get(path), snapshot.get(path)) {
        (Some(file), Some(saved)) => file.hash != saved.content_hash,
        (None, None) => false,
        _ => true,
    }
}

async fn download(repo: &RemoteRepo, head: &str, entries: Vec<RemoteEntry>) -> Result<(Vec<Download>, Vec<String>), ApiError> {
    let entries = remote::fill_sizes(repo, head, entries).await?;
    let mut downloads = Vec::new();
    let mut too_large = Vec::new();
    for entry in entries {
        if entry.size.is_some_and(|size| size > MAX_FILE_BYTES) {
            too_large.push(entry.path);
            continue;
        }
        match remote::read_blob(repo, &entry).await? {
            Some(content) => {
                let content = normalize_content(&entry.path, content);
                let hash = content_hash(&content);
                downloads.push(Download { path: entry.path, sha: entry.sha, content, hash });
            }
            None => too_large.push(entry.path),
        }
    }
    Ok((downloads, too_large))
}

fn added_bytes(actions: &[Action], local: &HashMap<String, LocalFile>) -> i64 {
    let previous_size = |path: &str| local.get(path).map(|file| file.stored_size).unwrap_or(0);
    actions
        .iter()
        .map(|action| match action {
            Action::Write(path, content) => content.len() as i64 - previous_size(path),
            Action::Remove(path) => -previous_size(path),
        })
        .sum()
}

async fn apply_actions(state: &AppState, project_id: &str, actions: Vec<Action>) -> Result<(), ApiError> {
    for action in actions {
        match action {
            Action::Write(path, content) => local_files::write(state, project_id, &path, content).await?,
            Action::Remove(path) => local_files::remove(state, project_id, &path).await?,
        }
    }
    Ok(())
}

pub async fn pull_into_project(state: &AppState, input: PullInput<'_>) -> Result<PullOutcome, ApiError> {
    let PullInput { project, link, head, entries, local, snapshot, resolution, remove_local_only, user_id } = input;
    let rules = link.rules();
    let changes = remote_changes(&entries, &snapshot, &rules);
    let remote_paths: HashSet<String> = entries.iter().map(|entry| entry.path.clone()).collect();
    let wanted: Vec<RemoteEntry> = entries
        .into_iter()
        .filter(|entry| matches!(changes.get(&entry.path), Some(ChangeKind::Added | ChangeKind::Modified)))
        .collect();
    let (downloads, too_large) = download(&link.remote(), head, wanted).await?;

    let mut outcome = PullOutcome { too_large, ..PullOutcome::default() };
    let mut actions = Vec::new();
    let mut update = SnapshotUpdate { saved: Vec::new(), removed: Vec::new() };

    for download in downloads {
        let unchanged = local.get(&download.path).is_some_and(|file| file.hash == download.hash);
        let conflict = !unchanged && is_locally_changed(&download.path, &local, &snapshot);
        if conflict && resolution == Resolution::Ask {
            outcome.conflicts.push(download.path);
            continue;
        }
        if !unchanged && (!conflict || resolution == Resolution::KeepRepository) {
            outcome.updated.push(download.path.clone());
            actions.push(Action::Write(download.path.clone(), download.content));
        }
        update.saved.push((download.path, SnapshotEntry { blob_sha: download.sha, content_hash: download.hash }));
    }

    for (path, _) in changes.iter().filter(|(_, kind)| **kind == ChangeKind::Deleted) {
        let exists_locally = local.contains_key(path);
        let conflict = exists_locally && is_locally_changed(path, &local, &snapshot);
        if conflict && resolution == Resolution::Ask {
            outcome.conflicts.push(path.clone());
            continue;
        }
        if exists_locally && (!conflict || resolution == Resolution::KeepRepository) {
            outcome.removed.push(path.clone());
            actions.push(Action::Remove(path.clone()));
        }
        update.removed.push(path.clone());
    }

    if remove_local_only {
        let mut local_only: Vec<&String> = local.keys().filter(|path| !remote_paths.contains(*path)).collect();
        local_only.sort();
        for path in local_only {
            outcome.removed.push(path.clone());
            actions.push(Action::Remove(path.clone()));
        }
    }

    if !outcome.conflicts.is_empty() {
        outcome.conflicts.sort();
        return Ok(outcome);
    }

    crate::accounts::storage::ensure_room(state, &project.owner_id, added_bytes(&actions, &local)).await?;
    let changed_files = !actions.is_empty();
    apply_actions(state, &project.id, actions).await?;
    snapshot::apply(state, &project.id, &update, head, &timestamp_now()).await?;

    if changed_files {
        local_files::repair_entrypoint(state, project).await?;
        notify_devices(state, &project.owner_id, DeviceEvent::project(&project.id)).await;
        crate::drive::activity::mark_project_edited(state, &project.id, Some(user_id)).await;
    }
    outcome.applied = true;
    Ok(outcome)
}

pub async fn pull(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<PullRequest>,
) -> Result<Json<PullResponse>, ApiError> {
    let access = editable_project(&state, &project_id, &user_id).await?;
    let link = require_link(&state, &project_id).await?;
    load_policy(&state).await.check_provider(link.provider)?;
    let _guard = lock_project(&project_id)?;

    let repo = link.remote();
    let head = remote::head(&repo).await?;
    if link.base_commit.as_deref() == Some(head.as_str()) {
        let outcome = PullOutcome { applied: true, ..PullOutcome::default() };
        return Ok(Json(PullResponse { link: link.view(access.is_owner), outcome }));
    }

    let rules = link.rules();
    let entries = remote::list_files(&repo, &head).await?;
    let outcome = pull_into_project(
        &state,
        PullInput {
            project: &access.project,
            link: &link,
            head: &head,
            entries,
            local: local_files::load(&state, &project_id, &rules).await?,
            snapshot: snapshot::load(&state, &project_id).await?,
            resolution: payload.resolution.unwrap_or_default(),
            remove_local_only: false,
            user_id: &user_id,
        },
    )
    .await?;

    let link = require_link(&state, &project_id).await?;
    Ok(Json(PullResponse { link: link.view(access.is_owner), outcome }))
}

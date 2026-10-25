use std::collections::{HashMap, HashSet};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use super::access::editable_project;
use super::file_rules::MAX_FILE_BYTES;
use super::links::{require_link, LinkView};
use super::local_files;
use super::locks::lock_project;
use super::policy::load_policy;
use super::remote::{self, Author, ChangeKind, FileChange};
use super::snapshot::{self, SnapshotEntry, SnapshotUpdate};
use super::sync_plan::local_changes;
use super::ApiError;
use crate::accounts::lookup::user_by_id;
use crate::accounts::signed_in::SignedInUser;
use crate::db::timestamp_now;
use crate::AppState;

const MAX_MESSAGE_LENGTH: usize = 5000;

#[derive(Deserialize)]
pub struct PushRequest {
    pub message: String,
    pub paths: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct PushResponse {
    pub link: LinkView,
    pub commit: String,
    pub pushed: Vec<String>,
}

fn clean_message(message: &str) -> Result<String, ApiError> {
    let message = message.trim();
    if message.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Write a short message that describes your changes".to_string()));
    }
    if message.chars().count() > MAX_MESSAGE_LENGTH {
        return Err((StatusCode::BAD_REQUEST, "The commit message is too long".to_string()));
    }
    Ok(message.to_string())
}

async fn commit_author(state: &AppState, user_id: &str) -> Result<Author, ApiError> {
    let user = user_by_id(state, user_id)
        .await?
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;
    Ok(Author { name: user.username, email: user.email })
}

pub async fn push(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<PushRequest>,
) -> Result<Json<PushResponse>, ApiError> {
    let access = editable_project(&state, &project_id, &user_id).await?;
    let link = require_link(&state, &project_id).await?;
    load_policy(&state).await.check_provider(link.provider)?;
    let message = clean_message(&payload.message)?;
    let _guard = lock_project(&project_id)?;

    let repo = link.remote();
    let head = remote::head(&repo).await?;
    let base_commit = link.base_commit.clone().unwrap_or_default();
    if head != base_commit {
        return Err((StatusCode::CONFLICT, "The repository has new commits. Pull them first.".to_string()));
    }

    let rules = link.rules();
    let mut local = local_files::load(&state, &project_id, &rules).await?;
    let saved = snapshot::load(&state, &project_id).await?;
    let selected: Option<HashSet<&String>> = payload.paths.as_ref().map(|paths| paths.iter().collect());
    let changes: Vec<(String, ChangeKind)> = local_changes(&local, &saved, &rules)
        .into_iter()
        .filter(|(path, _)| selected.as_ref().is_none_or(|selected| selected.contains(path)))
        .collect();
    if changes.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "There are no changes to commit".to_string()));
    }

    let remote_paths: HashSet<String> = remote::list_files(&repo, &head).await?.into_iter().map(|entry| entry.path).collect();
    for (path, kind) in &changes {
        if *kind == ChangeKind::Added && remote_paths.contains(path) {
            return Err((StatusCode::CONFLICT, format!("{path} exists in the repository but is not synced here. Rename it or exclude it.")));
        }
        if local.get(path).is_some_and(|file| file.content.len() as u64 > MAX_FILE_BYTES) {
            return Err((StatusCode::PAYLOAD_TOO_LARGE, format!("{path} is larger than 50 MB and cannot be pushed")));
        }
    }

    let hashes: HashMap<String, String> = changes
        .iter()
        .filter_map(|(path, _)| local.get(path).map(|file| (path.clone(), file.hash.clone())))
        .collect();
    let file_changes: Vec<FileChange> = changes
        .iter()
        .map(|(path, kind)| FileChange {
            path: path.clone(),
            kind: *kind,
            content: local.remove(path).map(|file| file.content).unwrap_or_default(),
        })
        .collect();

    let author = commit_author(&state, &user_id).await?;
    let commit = remote::commit(&repo, &base_commit, file_changes, &message, &author).await?;

    let pushed_shas: HashMap<String, String> = remote::list_files(&repo, &commit)
        .await?
        .into_iter()
        .filter(|entry| hashes.contains_key(&entry.path))
        .map(|entry| (entry.path, entry.sha))
        .collect();
    let mut update = SnapshotUpdate { saved: Vec::new(), removed: Vec::new() };
    for (path, kind) in &changes {
        match (kind, pushed_shas.get(path), hashes.get(path)) {
            (ChangeKind::Deleted, _, _) => update.removed.push(path.clone()),
            (_, Some(blob_sha), Some(content_hash)) => {
                update.saved.push((path.clone(), SnapshotEntry { blob_sha: blob_sha.clone(), content_hash: content_hash.clone() }));
            }
            _ => {}
        }
    }
    snapshot::apply(&state, &project_id, &update, &commit, &timestamp_now()).await?;

    let link = require_link(&state, &project_id).await?;
    Ok(Json(PushResponse {
        link: link.view(access.is_owner),
        commit,
        pushed: changes.into_iter().map(|(path, _)| path).collect(),
    }))
}

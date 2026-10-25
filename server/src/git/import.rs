use std::collections::{HashMap, HashSet};

use serde::Serialize;

use super::file_rules::{check_file_count, MAX_FILE_BYTES};
use super::links::GitLink;
use super::local_files::{self, LocalFile};
use super::pull::{pull_into_project, PullInput, Resolution};
use super::remote::{self, RemoteEntry};
use super::ApiError;
use crate::accounts::storage::{quota_for, usage_for};
use crate::models::Project;
use crate::AppState;

#[derive(Serialize)]
pub struct PreviewFile {
    pub path: String,
    pub size: Option<u64>,
}

#[derive(Serialize)]
pub struct ImportPreview {
    pub commit: String,
    pub files: Vec<PreviewFile>,
    pub too_large: Vec<PreviewFile>,
    pub ignored_count: usize,
    pub total_bytes: u64,
    pub needed_bytes: i64,
    pub available_bytes: Option<i64>,
    pub replaced: Vec<String>,
    pub removed: Vec<String>,
}

fn preview_file(entry: &RemoteEntry) -> PreviewFile {
    PreviewFile { path: entry.path.clone(), size: entry.size }
}

fn sorted(mut paths: Vec<String>) -> Vec<String> {
    paths.sort();
    paths
}

pub async fn preview_import(
    state: &AppState,
    owner_id: &str,
    local: &HashMap<String, LocalFile>,
    link: &GitLink,
    replace_project_files: bool,
) -> Result<(ImportPreview, Vec<RemoteEntry>), ApiError> {
    let repo = link.remote();
    let rules = link.rules();
    let commit = remote::head(&repo).await?;
    let listed = remote::list_files(&repo, &commit).await?;
    let listed_count = listed.len();
    let managed: Vec<RemoteEntry> = listed.into_iter().filter(|entry| rules.is_managed(&entry.path)).collect();
    let ignored_count = listed_count - managed.len();
    check_file_count(managed.len())?;
    let mut managed = remote::fill_sizes(&repo, &commit, managed).await?;
    managed.sort_by(|first, second| first.path.cmp(&second.path));

    let (too_large, importable): (Vec<&RemoteEntry>, Vec<&RemoteEntry>) =
        managed.iter().partition(|entry| entry.size.is_some_and(|size| size > MAX_FILE_BYTES));
    let remote_paths: HashSet<&str> = importable.iter().map(|entry| entry.path.as_str()).collect();

    let replaced: Vec<String> = local.keys().filter(|path| remote_paths.contains(path.as_str())).cloned().collect();
    let removed: Vec<String> = if replace_project_files {
        local.keys().filter(|path| !managed.iter().any(|entry| &entry.path == *path)).cloned().collect()
    } else {
        Vec::new()
    };

    let total_bytes: u64 = importable.iter().filter_map(|entry| entry.size).sum();
    let freed_bytes: i64 = replaced.iter().chain(removed.iter()).filter_map(|path| local.get(path)).map(|file| file.stored_size).sum();
    let quota = quota_for(state, owner_id).await;
    let available_bytes = if quota > 0 { Some(quota - usage_for(state, owner_id).await.total_size_bytes) } else { None };

    let preview = ImportPreview {
        commit,
        files: importable.iter().map(|entry| preview_file(entry)).collect(),
        too_large: too_large.iter().map(|entry| preview_file(entry)).collect(),
        ignored_count,
        total_bytes,
        needed_bytes: total_bytes as i64 - freed_bytes,
        available_bytes,
        replaced: sorted(replaced),
        removed: sorted(removed),
    };
    Ok((preview, managed))
}

pub async fn import_repository(
    state: &AppState,
    project: &Project,
    link: &GitLink,
    commit: &str,
    entries: Vec<RemoteEntry>,
    replace_project_files: bool,
    user_id: &str,
) -> Result<(), ApiError> {
    let local = local_files::load(state, &project.id, &link.rules()).await?;
    pull_into_project(
        state,
        PullInput {
            project,
            link,
            head: commit,
            entries,
            local,
            snapshot: HashMap::new(),
            resolution: Resolution::KeepRepository,
            remove_local_only: replace_project_files,
            user_id,
        },
    )
    .await?;
    Ok(())
}

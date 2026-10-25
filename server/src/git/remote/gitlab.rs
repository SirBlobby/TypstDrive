use axum::http::StatusCode;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde_json::{json, Value};

use super::http::{encode_segment, text_field, Client};
use super::{Author, ChangeKind, FileChange, RemoteEntry, RemoteRepo};
use crate::git::ApiError;

const SERVICE: &str = "GitLab";
const PAGE_SIZE: usize = 100;
const MAX_PAGES: usize = 200;
const SYMLINK_MODE: &str = "120000";

fn client(repo: &RemoteRepo) -> Client {
    let headers = repo.token.iter().map(|token| ("PRIVATE-TOKEN", token.clone())).collect();
    Client::new(SERVICE, format!("{}/api/v4/projects/{}", repo.url.web_base(), encode_segment(&repo.url.path)), headers)
}

fn unreadable() -> ApiError {
    (StatusCode::BAD_GATEWAY, format!("{SERVICE} sent an unreadable answer"))
}

pub fn head(repo: &RemoteRepo) -> Result<String, ApiError> {
    let body = client(repo).get_json(&format!("/repository/branches/{}", encode_segment(&repo.branch)))?;
    text_field(&body, "/commit/id").ok_or_else(unreadable)
}

pub fn list_files(repo: &RemoteRepo, commit: &str) -> Result<Vec<RemoteEntry>, ApiError> {
    let client = client(repo);
    let folder_filter = if repo.folder.is_empty() { String::new() } else { format!("&path={}", encode_segment(&repo.folder)) };
    let mut entries = Vec::new();
    let mut page = 1;
    loop {
        if page > MAX_PAGES {
            return Err((StatusCode::PAYLOAD_TOO_LARGE, "This folder has too many files to sync. Link a smaller folder.".to_string()));
        }
        let response = client.get(&format!("/repository/tree?ref={commit}&recursive=true&per_page={PAGE_SIZE}&page={page}{folder_filter}"), &[])?;
        let next_page = response.header("X-Next-Page").and_then(|value| value.trim().parse::<usize>().ok());
        let listing = response.into_json::<Value>().map_err(|_| unreadable())?;
        let listed = listing.as_array().ok_or_else(unreadable)?;
        entries.extend(
            listed
                .iter()
                .filter(|entry| entry.get("type").and_then(Value::as_str) == Some("blob"))
                .filter(|entry| entry.get("mode").and_then(Value::as_str) != Some(SYMLINK_MODE))
                .filter_map(|entry| {
                    let full_path = text_field(entry, "/path")?;
                    Some(RemoteEntry { path: repo.relative_path(&full_path)?, sha: text_field(entry, "/id")?, size: None })
                }),
        );
        match next_page {
            Some(next) if next > page => page = next,
            _ => break,
        }
    }
    Ok(entries)
}

pub fn fill_sizes(repo: &RemoteRepo, commit: &str, entries: Vec<RemoteEntry>) -> Result<Vec<RemoteEntry>, ApiError> {
    let client = client(repo);
    entries
        .into_iter()
        .map(|mut entry| {
            if entry.size.is_none() {
                let response = client.head(&format!("/repository/files/{}?ref={commit}", encode_segment(&repo.full_path(&entry.path))))?;
                entry.size = response.header("X-Gitlab-Size").and_then(|value| value.parse().ok());
            }
            Ok(entry)
        })
        .collect()
}

pub fn read_blob(repo: &RemoteRepo, sha: &str) -> Result<Option<Vec<u8>>, ApiError> {
    let client = client(repo);
    let response = client.get(&format!("/repository/blobs/{sha}/raw"), &[])?;
    client.read_limited(response)
}

fn action(repo: &RemoteRepo, change: &FileChange) -> Value {
    let file_path = repo.full_path(&change.path);
    match change.kind {
        ChangeKind::Deleted => json!({ "action": "delete", "file_path": file_path }),
        ChangeKind::Added | ChangeKind::Modified => json!({
            "action": if change.kind == ChangeKind::Added { "create" } else { "update" },
            "file_path": file_path,
            "content": BASE64.encode(&change.content),
            "encoding": "base64",
        }),
    }
}

pub fn commit(repo: &RemoteRepo, changes: &[FileChange], message: &str, author: &Author) -> Result<String, ApiError> {
    let actions: Vec<Value> = changes.iter().map(|change| action(repo, change)).collect();
    let body = client(repo).send_json(
        "POST",
        "/repository/commits",
        &json!({
            "branch": repo.branch,
            "commit_message": message,
            "author_name": author.name,
            "author_email": author.email,
            "actions": actions,
        }),
    )?;
    text_field(&body, "/id").ok_or_else(unreadable)
}

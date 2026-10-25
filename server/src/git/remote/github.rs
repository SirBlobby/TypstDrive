use axum::http::StatusCode;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde_json::{json, Value};

use super::http::{encode_segment, text_field, Client};
use super::{Author, ChangeKind, FileChange, RemoteEntry, RemoteRepo};
use crate::git::ApiError;

const SERVICE: &str = "GitHub";
const FILE_MODE: &str = "100644";
const SYMLINK_MODE: &str = "120000";

fn api_base(repo: &RemoteRepo) -> String {
    match repo.url.host_name() {
        "github.com" | "www.github.com" => "https://api.github.com".to_string(),
        _ => format!("{}/api/v3", repo.url.web_base()),
    }
}

fn client(repo: &RemoteRepo) -> Client {
    let mut headers = vec![("Accept", "application/vnd.github+json".to_string()), ("X-GitHub-Api-Version", "2022-11-28".to_string())];
    if let Some(token) = &repo.token {
        headers.push(("Authorization", format!("Bearer {token}")));
    }
    Client::new(SERVICE, format!("{}/repos/{}", api_base(repo), repo.url.path), headers)
}

fn branch_ref(repo: &RemoteRepo) -> String {
    let segments: Vec<String> = repo.branch.split('/').map(encode_segment).collect();
    format!("heads/{}", segments.join("/"))
}

fn unreadable() -> ApiError {
    (StatusCode::BAD_GATEWAY, format!("{SERVICE} sent an unreadable answer"))
}

fn commit_tree(client: &Client, commit: &str) -> Result<String, ApiError> {
    let body = client.get_json(&format!("/git/commits/{commit}"))?;
    text_field(&body, "/tree/sha").ok_or_else(unreadable)
}

fn folder_tree(client: &Client, repo: &RemoteRepo, commit: &str) -> Result<String, ApiError> {
    let mut tree_sha = commit_tree(client, commit)?;
    for segment in repo.folder.split('/').filter(|segment| !segment.is_empty()) {
        let listing = client.get_json(&format!("/git/trees/{tree_sha}"))?;
        let entries = listing.get("tree").and_then(Value::as_array).ok_or_else(unreadable)?;
        tree_sha = entries
            .iter()
            .find(|entry| entry.get("path").and_then(Value::as_str) == Some(segment) && entry.get("type").and_then(Value::as_str) == Some("tree"))
            .and_then(|entry| text_field(entry, "/sha"))
            .ok_or_else(|| (StatusCode::NOT_FOUND, format!("The folder {} does not exist on this branch", repo.folder)))?;
    }
    Ok(tree_sha)
}

pub fn head(repo: &RemoteRepo) -> Result<String, ApiError> {
    let body = client(repo).get_json(&format!("/git/ref/{}", branch_ref(repo)))?;
    text_field(&body, "/object/sha").ok_or_else(unreadable)
}

pub fn list_files(repo: &RemoteRepo, commit: &str) -> Result<Vec<RemoteEntry>, ApiError> {
    let client = client(repo);
    let tree_sha = folder_tree(&client, repo, commit)?;
    let listing = client.get_json(&format!("/git/trees/{tree_sha}?recursive=1"))?;
    if listing.get("truncated").and_then(Value::as_bool).unwrap_or(false) {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "This folder has too many files to sync. Link a smaller folder.".to_string()));
    }
    let entries = listing.get("tree").and_then(Value::as_array).ok_or_else(unreadable)?;
    Ok(entries
        .iter()
        .filter(|entry| entry.get("type").and_then(Value::as_str) == Some("blob"))
        .filter(|entry| entry.get("mode").and_then(Value::as_str) != Some(SYMLINK_MODE))
        .filter_map(|entry| {
            Some(RemoteEntry {
                path: text_field(entry, "/path")?,
                sha: text_field(entry, "/sha")?,
                size: entry.get("size").and_then(Value::as_u64),
            })
        })
        .collect())
}

pub fn read_blob(repo: &RemoteRepo, sha: &str) -> Result<Option<Vec<u8>>, ApiError> {
    let client = client(repo);
    let response = client.get(&format!("/git/blobs/{sha}"), &[("Accept", "application/vnd.github.raw+json")])?;
    client.read_limited(response)
}

fn tree_item(client: &Client, repo: &RemoteRepo, change: &FileChange) -> Result<Value, ApiError> {
    let path = repo.full_path(&change.path);
    if change.kind == ChangeKind::Deleted {
        return Ok(json!({ "path": path, "mode": FILE_MODE, "type": "blob", "sha": Value::Null }));
    }
    let blob = client.send_json("POST", "/git/blobs", &json!({ "content": BASE64.encode(&change.content), "encoding": "base64" }))?;
    let sha = text_field(&blob, "/sha").ok_or_else(unreadable)?;
    Ok(json!({ "path": path, "mode": FILE_MODE, "type": "blob", "sha": sha }))
}

pub fn commit(repo: &RemoteRepo, base_commit: &str, changes: &[FileChange], message: &str, author: &Author) -> Result<String, ApiError> {
    let client = client(repo);
    let base_tree = commit_tree(&client, base_commit)?;
    let items = changes.iter().map(|change| tree_item(&client, repo, change)).collect::<Result<Vec<Value>, ApiError>>()?;

    let tree = client.send_json("POST", "/git/trees", &json!({ "base_tree": base_tree, "tree": items }))?;
    let tree_sha = text_field(&tree, "/sha").ok_or_else(unreadable)?;

    let commit = client.send_json(
        "POST",
        "/git/commits",
        &json!({
            "message": message,
            "tree": tree_sha,
            "parents": [base_commit],
            "author": { "name": author.name, "email": author.email },
        }),
    )?;
    let commit_sha = text_field(&commit, "/sha").ok_or_else(unreadable)?;

    client.send_json("PATCH", &format!("/git/refs/{}", branch_ref(repo)), &json!({ "sha": commit_sha, "force": false }))?;
    Ok(commit_sha)
}

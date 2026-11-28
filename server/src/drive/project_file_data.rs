use std::collections::HashMap;

use chrono::{TimeDelta, Utc};

use crate::db::sql;
use crate::AppState;

const SETTLED_AFTER_SECONDS: i64 = 2;

pub struct StoredProjectFile {
    pub path: String,
    pub is_binary: bool,
    pub content: Vec<u8>,
}

fn cache_key(file_id: &str, updated_at: &str) -> String {
    format!("project-file:{file_id}:{updated_at}")
}

fn settled_before() -> String {
    (Utc::now() - TimeDelta::seconds(SETTLED_AFTER_SECONDS)).format("%Y-%m-%d %H:%M:%S").to_string()
}

async fn text_contents(state: &AppState, project_id: &str) -> HashMap<String, Vec<u8>> {
    sqlx::query_as::<_, (String, Option<Vec<u8>>)>(sql("SELECT id, content FROM project_files WHERE project_id = $1 AND kind <> 'binary'"))
        .bind(project_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(id, content)| (id, content.unwrap_or_default()))
        .collect()
}

async fn binary_contents(state: &AppState, project_id: &str) -> HashMap<String, Vec<u8>> {
    sqlx::query_as::<_, (String, Option<Vec<u8>>)>(sql("SELECT id, content FROM project_files WHERE project_id = $1 AND kind = 'binary'"))
        .bind(project_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(id, content)| (id, content.unwrap_or_default()))
        .collect()
}

pub async fn load(state: &AppState, project_id: &str) -> Vec<StoredProjectFile> {
    let listed = sqlx::query_as::<_, (String, String, String, Option<String>)>(sql("SELECT id, path, kind, updated_at FROM project_files WHERE project_id = $1"))
        .bind(project_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let mut cached_binaries: HashMap<String, Vec<u8>> = HashMap::new();
    let mut missing_binary = false;
    for (id, _, kind, updated_at) in &listed {
        if kind != "binary" {
            continue;
        }
        let Some(updated_at) = updated_at else {
            missing_binary = true;
            continue;
        };
        match state.file_storage.cached(&cache_key(id, updated_at)) {
            Some(data) => {
                cached_binaries.insert(id.clone(), data);
            }
            None => missing_binary = true,
        }
    }

    let mut texts = text_contents(state, project_id).await;
    let mut loaded_binaries = if missing_binary { binary_contents(state, project_id).await } else { HashMap::new() };
    let settled = settled_before();

    listed
        .into_iter()
        .map(|(id, path, kind, updated_at)| {
            let is_binary = kind == "binary";
            let content = if !is_binary {
                texts.remove(&id).unwrap_or_default()
            } else if let Some(data) = cached_binaries.remove(&id) {
                data
            } else {
                let data = loaded_binaries.remove(&id).unwrap_or_default();
                if let Some(updated_at) = updated_at.filter(|updated_at| *updated_at < settled) {
                    state.file_storage.remember(&cache_key(&id, &updated_at), data.clone());
                }
                data
            };
            StoredProjectFile { path, is_binary, content }
        })
        .collect()
}

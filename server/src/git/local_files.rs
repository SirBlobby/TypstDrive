use std::collections::HashMap;

use uuid::Uuid;

use super::file_rules::FileRules;
use super::{internal_error, ApiError};
use crate::db::sql;
use crate::drive::projects::{decode_text_blob, encode_text_blob, is_text_path, project_entrypoint_after_removal};
use crate::hashing::sha256_hex;
use crate::models::Project;
use crate::realtime::live_documents;
use crate::templates::create::mime_type;
use crate::AppState;

pub struct LocalFile {
    pub content: Vec<u8>,
    pub hash: String,
    pub stored_size: i64,
}

pub fn content_hash(content: &[u8]) -> String {
    sha256_hex(content)
}

pub fn normalize_content(path: &str, content: Vec<u8>) -> Vec<u8> {
    if is_text_path(path) {
        match String::from_utf8(content) {
            Ok(text) => text.into_bytes(),
            Err(error) => String::from_utf8_lossy(error.as_bytes()).into_owned().into_bytes(),
        }
    } else {
        content
    }
}

pub async fn load(state: &AppState, project_id: &str, rules: &FileRules) -> Result<HashMap<String, LocalFile>, ApiError> {
    let rows = sqlx::query_as::<_, (String, String, String, Option<Vec<u8>>)>(sql("SELECT id, path, kind, content FROM project_files WHERE project_id = $1"))
        .bind(project_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;

    let mut files = HashMap::new();
    for (file_id, path, kind, stored) in rows.into_iter().filter(|(_, path, _, _)| rules.is_managed(path)) {
        let stored = stored.unwrap_or_default();
        let stored_size = stored.len() as i64;
        let content = if kind == "binary" {
            stored
        } else {
            let room = live_documents::project_file_room(project_id, &file_id);
            match live_documents::live_text(state, &room).await {
                Some(text) => text.into_bytes(),
                None => decode_text_blob(&stored).into_bytes(),
            }
        };
        let hash = content_hash(&content);
        files.insert(path, LocalFile { content, hash, stored_size });
    }
    Ok(files)
}

async fn write_text(state: &AppState, project_id: &str, path: &str, text: &str) -> Result<(), ApiError> {
    let existing = sqlx::query_as::<_, (String, String, Option<Vec<u8>>)>(sql("SELECT id, kind, content FROM project_files WHERE project_id = $1 AND path = $2"))
        .bind(project_id)
        .bind(path)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;

    match existing {
        Some((file_id, kind, stored)) if kind == "text" => {
            let room = live_documents::project_file_room(project_id, &file_id);
            let blob = live_documents::store_text(state, &room, stored.as_deref(), text).await;
            sqlx::query(sql("UPDATE project_files SET content = $1, updated_at = $2 WHERE id = $3"))
                .bind(&blob)
                .bind(crate::db::timestamp_now())
                .bind(&file_id)
                .execute(&state.db)
                .await
                .map_err(internal_error)?;
            Ok(())
        }
        _ => upsert(state, project_id, path, "text", encode_text_blob(text)).await,
    }
}

async fn upsert(state: &AppState, project_id: &str, path: &str, kind: &str, content: Vec<u8>) -> Result<(), ApiError> {
    sqlx::query(sql(
        "INSERT INTO project_files (id, project_id, path, kind, content, mime_type) VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (project_id, path) DO UPDATE SET content = excluded.content, kind = excluded.kind, mime_type = excluded.mime_type",
    ))
    .bind(Uuid::new_v4().to_string())
    .bind(project_id)
    .bind(path)
    .bind(kind)
    .bind(content)
    .bind(mime_type(path))
    .execute(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(())
}

pub async fn write(state: &AppState, project_id: &str, path: &str, content: Vec<u8>) -> Result<(), ApiError> {
    if is_text_path(path) {
        write_text(state, project_id, path, &String::from_utf8_lossy(&content)).await
    } else {
        upsert(state, project_id, path, "binary", content).await
    }
}

pub async fn remove(state: &AppState, project_id: &str, path: &str) -> Result<(), ApiError> {
    sqlx::query(sql("DELETE FROM project_files WHERE project_id = $1 AND path = $2"))
        .bind(project_id)
        .bind(path)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(())
}

pub async fn repair_entrypoint(state: &AppState, project: &Project) -> Result<(), ApiError> {
    let paths: Vec<String> = sqlx::query_as::<_, (String,)>(sql("SELECT path FROM project_files WHERE project_id = $1"))
        .bind(&project.id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?
        .into_iter()
        .map(|(path,)| path)
        .collect();
    if paths.contains(&project.entrypoint) {
        return Ok(());
    }
    let entrypoint = project_entrypoint_after_removal(&project.entrypoint, &paths);
    sqlx::query(sql("UPDATE projects SET entrypoint = $1 WHERE id = $2"))
        .bind(&entrypoint)
        .bind(&project.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(())
}

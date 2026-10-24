use axum::http::StatusCode;
use uuid::Uuid;

use super::{internal_error, ApiError};
use crate::db::{sql, timestamp_now};
use crate::desktop::devices::{notify_devices, DeviceEvent};
use crate::drive::project_paths::clean_project_path;
use crate::drive::projects::{decode_text_blob, encode_text_blob};
use crate::models::{Document, Project};
use crate::realtime::live_documents;
use crate::sharing::access;
use crate::AppState;

const BIBLIOGRAPHY_EXTENSIONS: &[&str] = &[".bib", ".yml", ".yaml"];

pub enum Target {
    Document(Document),
    Project(Project),
}

impl Target {
    pub fn id(&self) -> &str {
        match self {
            Self::Document(document) => &document.id,
            Self::Project(project) => &project.id,
        }
    }

    fn owner_id(&self) -> &str {
        match self {
            Self::Document(document) => &document.owner_id,
            Self::Project(project) => &project.owner_id,
        }
    }
}

pub fn is_bibliography_file(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    BIBLIOGRAPHY_EXTENSIONS.iter().any(|extension| lower.ends_with(extension))
}

pub fn clean_file_name(target: &Target, name: &str) -> Result<String, ApiError> {
    let name = name.trim();
    if !is_bibliography_file(name) {
        return Err((StatusCode::BAD_REQUEST, "Bibliography files must end in .bib, .yml or .yaml".to_string()));
    }
    match target {
        Target::Project(_) => clean_project_path(name),
        Target::Document(_) if name.contains(['/', '\\']) || name.chars().any(char::is_control) => {
            Err((StatusCode::BAD_REQUEST, "File names cannot contain slashes".to_string()))
        }
        Target::Document(_) => Ok(name.to_string()),
    }
}

pub async fn editable_target(state: &AppState, kind: &str, item_id: &str, user_id: &Option<String>) -> Result<Target, ApiError> {
    let not_found = || (StatusCode::NOT_FOUND, "Not found".to_string());
    let (target, can_edit) = match kind {
        "document" => {
            let (document, role) = access::document_role(state, item_id, user_id).await.ok_or_else(not_found)?;
            (Target::Document(document), role.can_edit())
        }
        "project" => {
            let (project, role) = access::project_access(state, item_id, user_id).await.ok_or_else(not_found)?;
            (Target::Project(project), role.can_edit())
        }
        _ => return Err((StatusCode::BAD_REQUEST, "Citations can only be added to documents and projects".to_string())),
    };
    if !can_edit || user_id.is_none() {
        return Err((StatusCode::FORBIDDEN, "You need edit access to change the bibliography".to_string()));
    }
    Ok(target)
}

pub async fn bibliography_files(state: &AppState, target: &Target) -> Result<Vec<String>, ApiError> {
    let names: Vec<String> = match target {
        Target::Document(document) => sqlx::query_as::<_, (String,)>(sql("SELECT name FROM files WHERE document_id = $1 ORDER BY name ASC"))
            .bind(&document.id)
            .fetch_all(&state.db)
            .await
            .map_err(internal_error)?
            .into_iter()
            .map(|(name,)| name)
            .collect(),
        Target::Project(project) => sqlx::query_as::<_, (String,)>(sql("SELECT path FROM project_files WHERE project_id = $1 AND kind = 'text' ORDER BY path ASC"))
            .bind(&project.id)
            .fetch_all(&state.db)
            .await
            .map_err(internal_error)?
            .into_iter()
            .map(|(path,)| path)
            .collect(),
    };
    let mut names: Vec<String> = names.into_iter().filter(|name| is_bibliography_file(name)).collect();
    names.dedup();
    Ok(names)
}

async fn document_file(state: &AppState, document: &Document, name: &str) -> Result<Option<(String, Vec<u8>, Option<String>, i64)>, ApiError> {
    sqlx::query_as::<_, (String, Vec<u8>, Option<String>, i64)>(sql(
        "SELECT id, data, storage_key, size_bytes FROM files WHERE document_id = $1 AND name = $2 ORDER BY created_at ASC LIMIT 1",
    ))
    .bind(&document.id)
    .bind(name)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)
}

async fn project_file(state: &AppState, project: &Project, path: &str) -> Result<Option<(String, String, Option<Vec<u8>>)>, ApiError> {
    sqlx::query_as::<_, (String, String, Option<Vec<u8>>)>(sql("SELECT id, kind, content FROM project_files WHERE project_id = $1 AND path = $2"))
        .bind(&project.id)
        .bind(path)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)
}

pub async fn read_text(state: &AppState, target: &Target, name: &str) -> Result<Option<String>, ApiError> {
    match target {
        Target::Document(document) => match document_file(state, document, name).await? {
            Some((_, data, storage_key, _)) => {
                let bytes = state.file_storage.load(data, storage_key).await?;
                Ok(Some(String::from_utf8_lossy(&bytes).to_string()))
            }
            None => Ok(None),
        },
        Target::Project(project) => match project_file(state, project, name).await? {
            Some((_, kind, _)) if kind != "text" => Err((StatusCode::BAD_REQUEST, format!("{name} is not a text file"))),
            Some((file_id, _, content)) => {
                let room = live_documents::project_file_room(&project.id, &file_id);
                match live_documents::live_text(state, &room).await {
                    Some(text) => Ok(Some(text)),
                    None => Ok(Some(decode_text_blob(&content.unwrap_or_default()))),
                }
            }
            None => Ok(None),
        },
    }
}

async fn write_document_file(state: &AppState, document: &Document, name: &str, text: &str, user_id: &str) -> Result<(), ApiError> {
    let data = text.as_bytes().to_vec();
    let existing = document_file(state, document, name).await?;
    let previous_size = existing.as_ref().map(|(_, _, _, size)| *size).unwrap_or(0);
    crate::accounts::storage::ensure_room(state, &document.owner_id, data.len() as i64 - previous_size).await?;
    let stored = state.file_storage.store(data).await?;

    match existing {
        Some((file_id, _, _, _)) => {
            sqlx::query(sql("UPDATE files SET data = $1, storage_key = $2, size_bytes = $3, updated_at = $4, last_edited_by = $5 WHERE id = $6"))
                .bind(&stored.inline_data)
                .bind(&stored.storage_key)
                .bind(stored.size_bytes)
                .bind(timestamp_now())
                .bind(user_id)
                .bind(&file_id)
                .execute(&state.db)
                .await
                .map_err(internal_error)?;
        }
        None => {
            sqlx::query(sql(
                "INSERT INTO files (id, owner_id, document_id, folder_id, name, mime_type, data, size_bytes, storage_key) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            ))
            .bind(Uuid::new_v4().to_string())
            .bind(&document.owner_id)
            .bind(&document.id)
            .bind(&document.folder_id)
            .bind(name)
            .bind("text/plain")
            .bind(&stored.inline_data)
            .bind(stored.size_bytes)
            .bind(&stored.storage_key)
            .execute(&state.db)
            .await
            .map_err(internal_error)?;
        }
    }
    Ok(())
}

async fn write_project_file(state: &AppState, project: &Project, path: &str, text: &str, user_id: &str) -> Result<(), ApiError> {
    let existing = project_file(state, project, path).await?;
    match existing {
        Some((_, kind, _)) if kind != "text" => return Err((StatusCode::BAD_REQUEST, format!("{path} is not a text file"))),
        Some((file_id, _, content)) => {
            let room = live_documents::project_file_room(&project.id, &file_id);
            let previous_size = content.as_ref().map(Vec::len).unwrap_or(0) as i64;
            let blob = live_documents::store_text(state, &room, content.as_deref(), text).await;
            crate::accounts::storage::ensure_room(state, &project.owner_id, blob.len() as i64 - previous_size).await?;
            sqlx::query(sql("UPDATE project_files SET content = $1 WHERE id = $2 AND project_id = $3"))
                .bind(&blob)
                .bind(&file_id)
                .bind(&project.id)
                .execute(&state.db)
                .await
                .map_err(internal_error)?;
        }
        None => {
            let blob = encode_text_blob(text);
            crate::accounts::storage::ensure_room(state, &project.owner_id, blob.len() as i64).await?;
            sqlx::query(sql("INSERT INTO project_files (id, project_id, path, kind, content, mime_type) VALUES ($1, $2, $3, 'text', $4, 'text/plain')"))
                .bind(Uuid::new_v4().to_string())
                .bind(&project.id)
                .bind(path)
                .bind(&blob)
                .execute(&state.db)
                .await
                .map_err(internal_error)?;
        }
    }
    crate::drive::activity::mark_project_edited(state, &project.id, Some(user_id)).await;
    Ok(())
}

pub async fn write_text(state: &AppState, target: &Target, name: &str, text: &str, user_id: &str) -> Result<(), ApiError> {
    match target {
        Target::Document(document) => write_document_file(state, document, name, text, user_id).await?,
        Target::Project(project) => write_project_file(state, project, name, text, user_id).await?,
    }
    let event = match target {
        Target::Document(document) => DeviceEvent::document(&document.id),
        Target::Project(project) => DeviceEvent::project(&project.id),
    };
    notify_devices(state, target.owner_id(), event).await;
    Ok(())
}

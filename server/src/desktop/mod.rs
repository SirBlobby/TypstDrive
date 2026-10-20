use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::sharing::access::AccessRole;
use crate::{
    desktop::devices::{notify_devices, DeviceEvent},
    models::{Project, User},
    drive::projects::{decode_text_blob, encode_text_blob},
    AppState,
};
use crate::db::sql;

pub mod devices;
pub mod session;
pub mod projects;
pub mod project_files;
pub mod folders;
pub mod documents;
pub mod files;

const TEXT_EXTENSIONS: [&str; 10] = [
    ".typ", ".toml", ".bib", ".csl", ".yml", ".yaml", ".json", ".md", ".txt", ".csv",
];

pub const MIN_DESKTOP_VERSION: &str = "1.0.0";

fn parse_version(version: &str) -> (u32, u32, u32) {
    let mut parts = version.trim().split('.').map(|part| part.parse::<u32>().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

fn version_at_least(actual: &str, required: &str) -> bool {
    parse_version(actual) >= parse_version(required)
}

#[derive(Serialize)]
pub struct ServerVersionInfo {
    pub server_version: String,
    pub min_desktop_version: String,
}

pub async fn version_info() -> Json<ServerVersionInfo> {
    Json(ServerVersionInfo {
        server_version: env!("CARGO_PKG_VERSION").to_string(),
        min_desktop_version: MIN_DESKTOP_VERSION.to_string(),
    })
}

fn is_text_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    TEXT_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

fn content_hash(bytes: &[u8]) -> String {
    crate::hashing::sha256_hex(bytes)
}

pub(crate) fn hash_token(token: &str) -> String {
    crate::hashing::sha256_hex(token.as_bytes())
}

fn generate_token() -> String {
    format!(
        "tdd_{}{}",
        Uuid::new_v4().to_string().replace("-", ""),
        Uuid::new_v4().to_string().replace("-", "")
    )
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.starts_with("Bearer "))
        .map(|value| value[7..].to_string())
}

pub async fn authenticate(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<String, (StatusCode, String)> {
    let token = bearer_token(headers).ok_or((
        StatusCode::UNAUTHORIZED,
        "Missing Authorization header. Use: Authorization: Bearer <device-token>".to_string(),
    ))?;

    let row = sqlx::query_as::<_, (String, String)>(
        sql("SELECT id, user_id FROM device_tokens WHERE token_hash = $1"),
    )
    .bind(hash_token(&token))
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (token_id, user_id) =
        row.ok_or((StatusCode::UNAUTHORIZED, "Invalid device token".to_string()))?;

    let _ = sqlx::query(sql("UPDATE device_tokens SET last_used_at = $1 WHERE id = $2"))
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(&token_id)
        .execute(&state.db)
        .await;

    Ok(user_id)
}

pub(crate) async fn user_id_for_token(state: &AppState, token: &str) -> Option<String> {
    device_token_for(state, token).await.map(|(_, user_id)| user_id)
}

pub(crate) async fn device_token_for(state: &AppState, token: &str) -> Option<(String, String)> {
    let row = sqlx::query_as::<_, (String, String)>(
        sql("SELECT id, user_id FROM device_tokens WHERE token_hash = $1"),
    )
    .bind(hash_token(token))
    .fetch_optional(&state.db)
    .await
    .ok()??;

    let (token_id, user_id) = row;

    let _ = sqlx::query(sql("UPDATE device_tokens SET last_used_at = $1 WHERE id = $2"))
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(&token_id)
        .execute(&state.db)
        .await;

    Some((token_id, user_id))
}

type DesktopError = (StatusCode, String);

async fn project_with_role(
    state: &AppState,
    project_id: &str,
    user_id: &str,
) -> Result<(Project, AccessRole), DesktopError> {
    crate::sharing::access::project_access(state, project_id, &Some(user_id.to_string()))
        .await
        .ok_or((StatusCode::NOT_FOUND, "Project not found".to_string()))
}

async fn readable_project(state: &AppState, project_id: &str, user_id: &str) -> Result<Project, DesktopError> {
    Ok(project_with_role(state, project_id, user_id).await?.0)
}

async fn writable_project(state: &AppState, project_id: &str, user_id: &str) -> Result<Project, DesktopError> {
    let (project, role) = project_with_role(state, project_id, user_id).await?;
    if !role.can_edit() {
        return Err((StatusCode::FORBIDDEN, "Read-only access".to_string()));
    }
    Ok(project)
}

async fn document_owner(state: &AppState, document_id: &str) -> Option<String> {
    sqlx::query_as::<_, (String,)>(sql("SELECT owner_id FROM documents WHERE id = $1"))
        .bind(document_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .map(|(owner_id,)| owner_id)
}

async fn owned_folder(
    state: &AppState,
    folder_id: &str,
    user_id: &str,
) -> Result<(), (StatusCode, String)> {
    let row = sqlx::query_as::<_, (i64,)>(
        sql("SELECT COUNT(*) FROM folders WHERE id = $1 AND owner_id = $2"),
    )
    .bind(folder_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if row.0 == 0 {
        return Err((StatusCode::NOT_FOUND, "Folder not found".to_string()));
    }

    Ok(())
}

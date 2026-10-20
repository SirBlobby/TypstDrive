mod history;
mod merge;
pub mod retention;

pub use history::save_when_leaving;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use crate::accounts::signed_in::SignedInUser;
use serde::{Deserialize, Serialize};

use crate::db::sql;
use crate::drive::projects::encode_text_blob;
use crate::realtime::live_documents;
use crate::sharing::access;
use crate::AppState;
use history::{find_version, list_versions, save_version, NewVersion, VersionView, KIND_AUTO, KIND_MANUAL, KIND_RESTORE};

type ApiError = (StatusCode, String);

#[derive(Deserialize)]
pub struct SaveVersionRequest {
    pub label: Option<String>,
}

#[derive(Deserialize)]
pub struct RenameVersionRequest {
    pub label: Option<String>,
}

#[derive(Serialize)]
pub struct SaveVersionResponse {
    pub created: bool,
    pub version: Option<VersionView>,
}

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn clean_label(label: Option<String>) -> Option<String> {
    label.map(|label| label.trim().chars().take(120).collect::<String>()).filter(|label| !label.is_empty())
}

async fn require_editor(state: &AppState, document_id: &str, user_id: &str) -> Result<(), ApiError> {
    let (_, role) = access::document_role(state, document_id, &Some(user_id.to_string()))
        .await
        .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;
    if role.can_edit() {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "Only editors can change version history".to_string()))
    }
}

pub async fn get_versions(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<Json<Vec<VersionView>>, ApiError> {
    access::document_role(&state, &document_id, &Some(user_id)).await.ok_or_else(access::forbidden)?;
    Ok(Json(list_versions(&state, &document_id).await.map_err(internal_error)?))
}

pub async fn create_version(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<SaveVersionRequest>,
) -> Result<Json<SaveVersionResponse>, ApiError> {
    require_editor(&state, &document_id, &user_id).await?;
    let content = history::current_text(&state, &document_id)
        .await
        .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;

    let label = clean_label(payload.label);
    let new_version = NewVersion { document_id: &document_id, author_id: &user_id, content: &content, kind: KIND_MANUAL, label: label.as_deref() };
    let created_id = save_version(&state, new_version).await.map_err(internal_error)?;

    let version = match &created_id {
        Some(version_id) => find_version(&state, &document_id, version_id).await.map_err(internal_error)?,
        None => None,
    };
    Ok(Json(SaveVersionResponse { created: created_id.is_some(), version }))
}

pub async fn rename_version(
    State(state): State<AppState>,
    Path((document_id, version_id)): Path<(String, String)>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<RenameVersionRequest>,
) -> Result<StatusCode, ApiError> {
    require_editor(&state, &document_id, &user_id).await?;
    let result = sqlx::query(sql("UPDATE document_versions SET label = $1 WHERE id = $2 AND document_id = $3"))
        .bind(clean_label(payload.label))
        .bind(&version_id)
        .bind(&document_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Version not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn restore_version(
    State(state): State<AppState>,
    Path((document_id, version_id)): Path<(String, String)>,
    SignedInUser(user_id): SignedInUser,
) -> Result<StatusCode, ApiError> {
    require_editor(&state, &document_id, &user_id).await?;
    let version = find_version(&state, &document_id, &version_id)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Version not found".to_string()))?;

    if let Some(current) = history::current_text(&state, &document_id).await {
        let backup = NewVersion { document_id: &document_id, author_id: &user_id, content: &current, kind: KIND_AUTO, label: Some("Before restoring an older version") };
        save_version(&state, backup).await.map_err(internal_error)?;
    }

    sqlx::query(sql("UPDATE documents SET content = $1, updated_at = $2 WHERE id = $3"))
        .bind(encode_text_blob(&version.content))
        .bind(crate::db::timestamp_now())
        .bind(&document_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    live_documents::replace_text(&state, &live_documents::document_room(&document_id), &version.content).await;
    crate::drive::activity::mark_document_edited(&state, &document_id, Some(&user_id)).await;

    let restored_label = format!("Restored from {}", version.created_at.get(..16).unwrap_or(&version.created_at));
    let restored = NewVersion { document_id: &document_id, author_id: &user_id, content: &version.content, kind: KIND_RESTORE, label: Some(&restored_label) };
    save_version(&state, restored).await.map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

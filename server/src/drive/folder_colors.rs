use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::accounts::signed_in::SignedInUser;
use crate::db::sql;
use crate::desktop::devices::{notify_devices, DeviceEvent};
use crate::AppState;

type ApiError = (StatusCode, String);

#[derive(Deserialize)]
pub struct FolderColorRequest {
    pub color: Option<String>,
}

#[derive(Serialize)]
pub struct FolderColorResponse {
    pub id: String,
    pub color: Option<String>,
}

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn normalized_color(color: Option<String>) -> Result<Option<String>, ApiError> {
    let Some(color) = color.map(|value| value.trim().to_lowercase()).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let is_hex_color = color.len() == 7
        && color.starts_with('#')
        && color[1..].chars().all(|character| character.is_ascii_hexdigit());
    if !is_hex_color {
        return Err((StatusCode::BAD_REQUEST, "Folder colors must look like #1a2b3c".to_string()));
    }
    Ok(Some(color))
}

pub async fn set_folder_color(
    State(state): State<AppState>,
    Path(id): Path<String>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<FolderColorRequest>,
) -> Result<Json<FolderColorResponse>, ApiError> {
    apply_folder_color(&state, id, &user_id, payload.color).await
}

pub async fn apply_folder_color(
    state: &AppState,
    id: String,
    owner_id: &str,
    requested_color: Option<String>,
) -> Result<Json<FolderColorResponse>, ApiError> {
    let color = normalized_color(requested_color)?;

    let owned_folder = sqlx::query_as::<_, (String,)>(sql("SELECT id FROM folders WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL"))
        .bind(&id)
        .bind(owner_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;
    if owned_folder.is_none() {
        return Err((StatusCode::NOT_FOUND, "Folder not found".to_string()));
    }

    sqlx::query(sql("UPDATE folders SET color = $1 WHERE id = $2 AND owner_id = $3"))
        .bind(&color)
        .bind(&id)
        .bind(owner_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    notify_devices(state, owner_id, DeviceEvent::structure()).await;

    Ok(Json(FolderColorResponse { id, color }))
}

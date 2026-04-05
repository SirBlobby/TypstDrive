use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    models::{Folder, CreateFolderRequest},
    AppState,
};

#[derive(Deserialize)]
pub struct ListFoldersQuery {
    pub parent_id: Option<String>,
}

pub async fn list_folders(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Query(query): Query<ListFoldersQuery>,
) -> Result<Json<Vec<Folder>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let folders = if let Some(parent_id) = query.parent_id {
        sqlx::query_as::<_, Folder>(
            "SELECT id, owner_id, parent_id, name, created_at FROM folders WHERE owner_id = ? AND parent_id = ? ORDER BY name ASC"
        )
        .bind(&user_id)
        .bind(&parent_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        sqlx::query_as::<_, Folder>(
            "SELECT id, owner_id, parent_id, name, created_at FROM folders WHERE owner_id = ? AND parent_id IS NULL ORDER BY name ASC"
        )
        .bind(&user_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    Ok(Json(folders))
}

pub async fn create_folder(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<CreateFolderRequest>,
) -> Result<Json<Folder>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let folder_id = Uuid::new_v4().to_string();

    let folder = sqlx::query_as::<_, Folder>(
        "INSERT INTO folders (id, owner_id, parent_id, name) VALUES (?, ?, ?, ?) RETURNING id, owner_id, parent_id, name, created_at"
    )
    .bind(&folder_id)
    .bind(&user_id)
    .bind(&payload.parent_id)
    .bind(&payload.name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(folder))
}

pub async fn delete_folder(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    
    
    let result = sqlx::query("DELETE FROM folders WHERE id = ? AND owner_id = ?")
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Folder not found or unauthorized".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct UpdateFolderRequest {
    pub name: String,
}

pub async fn update_folder(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    Json(payload): Json<UpdateFolderRequest>,
) -> Result<Json<Folder>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let folder = sqlx::query_as::<_, Folder>(
        "UPDATE folders SET name = ? WHERE id = ? AND owner_id = ? RETURNING id, owner_id, parent_id, name, created_at"
    )
    .bind(&payload.name)
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match folder {
        Some(f) => Ok(Json(f)),
        None => Err((StatusCode::NOT_FOUND, "Folder not found".to_string())),
    }
}


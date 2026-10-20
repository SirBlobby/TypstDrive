use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;

use crate::db::{sql, timestamp_now};
use crate::models::File;
use crate::sharing::access::{document_role, inherited_role};
use crate::AppState;

type ApiError = (StatusCode, String);

struct ExistingFile {
    owner_id: String,
    document_id: Option<String>,
    folder_id: Option<String>,
    size_bytes: i64,
}

fn internal_error(error: impl ToString) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn not_found() -> ApiError {
    (StatusCode::NOT_FOUND, "File not found".to_string())
}

async fn existing_file(state: &AppState, file_id: &str) -> Result<ExistingFile, ApiError> {
    let (owner_id, document_id, folder_id, size_bytes) = sqlx::query_as::<_, (String, Option<String>, Option<String>, i64)>(sql(
        "SELECT owner_id, document_id, folder_id, size_bytes FROM files WHERE id = $1",
    ))
    .bind(file_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or_else(not_found)?;
    Ok(ExistingFile { owner_id, document_id, folder_id, size_bytes })
}

async fn can_replace(state: &AppState, file: &ExistingFile, user_id: &str) -> bool {
    if file.owner_id == user_id {
        return true;
    }
    if crate::teams::content_role(state, &file.owner_id, user_id).await.is_some_and(|role| role.can_edit()) {
        return true;
    }
    let user = Some(user_id.to_string());
    if inherited_role(state, &file.folder_id, &file.owner_id, &user).await.is_some_and(|role| role.can_edit()) {
        return true;
    }
    match file.document_id.as_deref() {
        Some(document_id) => document_role(state, document_id, &user).await.is_some_and(|(_, role)| role.can_edit()),
        None => false,
    }
}

pub async fn replace_file_data(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    mut multipart: Multipart,
) -> Result<Json<File>, ApiError> {
    let user_id = jar
        .get("session_user_id")
        .map(|cookie| cookie.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let file = existing_file(&state, &id).await?;
    if !can_replace(&state, &file, &user_id).await {
        return Err(not_found());
    }

    let field = multipart
        .next_field()
        .await
        .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))?
        .ok_or((StatusCode::BAD_REQUEST, "No file provided".to_string()))?;
    let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
    let data = field
        .bytes()
        .await
        .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))?
        .to_vec();
    if data.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "The replacement file is empty".to_string()));
    }

    crate::accounts::storage::ensure_room(&state, &file.owner_id, data.len() as i64 - file.size_bytes).await?;
    let stored = state.file_storage.store(data).await?;

    sqlx::query(sql(
        "UPDATE files SET data = $1, storage_key = $2, size_bytes = $3, mime_type = $4, updated_at = $5, last_edited_by = $6 WHERE id = $7",
    ))
    .bind(&stored.inline_data)
    .bind(&stored.storage_key)
    .bind(stored.size_bytes)
    .bind(&content_type)
    .bind(timestamp_now())
    .bind(&user_id)
    .bind(&id)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    let updated = sqlx::query_as::<_, File>(sql(
        "SELECT id, owner_id, document_id, folder_id, name, mime_type, created_at, size_bytes, updated_at FROM files WHERE id = $1",
    ))
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(Json(updated))
}

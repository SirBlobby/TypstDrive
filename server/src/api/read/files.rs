use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::Json;
use serde::Serialize;
use sqlx::FromRow;

use super::{database_error, not_found};
use crate::api::authentication::{authenticate, ApiScope};
use crate::db::sql;
use crate::AppState;

#[derive(Serialize, FromRow)]
pub struct FileSummary {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub folder_id: Option<String>,
    pub document_id: Option<String>,
    pub size_bytes: i64,
    pub created_at: String,
    pub updated_at: Option<String>,
}

pub async fn list_files(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<FileSummary>>, Response> {
    let caller = authenticate(&state, &headers, ApiScope::Read).await?;
    sqlx::query_as::<_, FileSummary>(sql(
        "SELECT id, name, mime_type, folder_id, document_id, size_bytes, created_at, updated_at \
         FROM files WHERE owner_id = $1 ORDER BY name ASC",
    ))
    .bind(&caller.account_id)
    .fetch_all(&state.db)
    .await
    .map(Json)
    .map_err(database_error)
}

pub async fn get_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(file_id): Path<String>,
) -> Result<Response, Response> {
    let caller = authenticate(&state, &headers, ApiScope::Read).await?;
    let (mime_type, data, storage_key) = sqlx::query_as::<_, (String, Vec<u8>, Option<String>)>(sql(
        "SELECT mime_type, data, storage_key FROM files WHERE id = $1 AND owner_id = $2",
    ))
    .bind(&file_id)
    .bind(&caller.account_id)
    .fetch_optional(&state.db)
    .await
    .map_err(database_error)?
    .ok_or_else(|| not_found("File"))?;

    let data = state.file_storage.load(data, storage_key).await.map_err(|_| not_found("File"))?;
    Ok(crate::user_content::response(&mime_type, data))
}

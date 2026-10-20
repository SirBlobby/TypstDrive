use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use axum::Json;
use serde::Serialize;
use sqlx::FromRow;

use super::database_error;
use crate::api::authentication::{authenticate, ApiScope};
use crate::db::sql;
use crate::AppState;

#[derive(Serialize, FromRow)]
pub struct FolderSummary {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub color: Option<String>,
    pub created_at: String,
}

pub async fn list_folders(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<FolderSummary>>, Response> {
    let caller = authenticate(&state, &headers, ApiScope::Read).await?;
    sqlx::query_as::<_, FolderSummary>(sql(
        "SELECT id, parent_id, name, color, created_at FROM folders WHERE owner_id = $1 ORDER BY name ASC",
    ))
    .bind(&caller.account_id)
    .fetch_all(&state.db)
    .await
    .map(Json)
    .map_err(database_error)
}

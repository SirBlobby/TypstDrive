use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use super::policy::load_policy;
use super::{internal_error, secrets, zotero, ApiError};
use crate::accounts::signed_in::SignedInUser;
use crate::db::sql;
use crate::AppState;

pub struct ZoteroCredentials {
    pub zotero_user_id: String,
    pub username: String,
    pub api_key: String,
}

#[derive(Serialize)]
pub struct ZoteroStatus {
    pub enabled: bool,
    pub connected: bool,
    pub username: Option<String>,
}

#[derive(Deserialize)]
pub struct ConnectRequest {
    pub api_key: String,
}

#[derive(Deserialize)]
pub struct CollectionsQuery {
    pub library: String,
}

pub async fn credentials(state: &AppState, user_id: &str) -> Result<Option<ZoteroCredentials>, ApiError> {
    let row = sqlx::query_as::<_, (String, String, String)>(sql("SELECT zotero_user_id, username, api_key_sealed FROM zotero_accounts WHERE user_id = $1"))
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(row.and_then(|(zotero_user_id, username, sealed)| {
        secrets::open(&state.key, &sealed).map(|api_key| ZoteroCredentials { zotero_user_id, username, api_key })
    }))
}

pub async fn require_credentials(state: &AppState, user_id: &str) -> Result<ZoteroCredentials, ApiError> {
    load_policy(state).await.check_zotero()?;
    credentials(state, user_id)
        .await?
        .ok_or((StatusCode::PRECONDITION_FAILED, "Connect your Zotero account in Settings first".to_string()))
}

pub async fn status(State(state): State<AppState>, SignedInUser(user_id): SignedInUser) -> Result<Json<ZoteroStatus>, ApiError> {
    let enabled = load_policy(&state).await.zotero_enabled;
    let account = credentials(&state, &user_id).await?;
    Ok(Json(ZoteroStatus { enabled, connected: account.is_some(), username: account.map(|account| account.username) }))
}

pub async fn connect(
    State(state): State<AppState>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<ConnectRequest>,
) -> Result<Json<ZoteroStatus>, ApiError> {
    load_policy(&state).await.check_zotero()?;
    let api_key = payload.api_key.trim().to_string();
    let info = zotero::key_info(&api_key).await?;

    sqlx::query(sql(
        "INSERT INTO zotero_accounts (user_id, zotero_user_id, username, api_key_sealed) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (user_id) DO UPDATE SET zotero_user_id = excluded.zotero_user_id, username = excluded.username, api_key_sealed = excluded.api_key_sealed",
    ))
    .bind(&user_id)
    .bind(&info.user_id)
    .bind(&info.username)
    .bind(secrets::seal(&state.key, &api_key))
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(Json(ZoteroStatus { enabled: true, connected: true, username: Some(info.username) }))
}

pub async fn disconnect(State(state): State<AppState>, SignedInUser(user_id): SignedInUser) -> Result<StatusCode, ApiError> {
    sqlx::query(sql("DELETE FROM zotero_accounts WHERE user_id = $1"))
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn libraries(State(state): State<AppState>, SignedInUser(user_id): SignedInUser) -> Result<Json<Vec<zotero::Library>>, ApiError> {
    let account = require_credentials(&state, &user_id).await?;
    Ok(Json(zotero::libraries(&account.zotero_user_id, &account.username, &account.api_key).await?))
}

pub async fn collections(
    State(state): State<AppState>,
    SignedInUser(user_id): SignedInUser,
    Query(query): Query<CollectionsQuery>,
) -> Result<Json<Vec<zotero::Collection>>, ApiError> {
    let account = require_credentials(&state, &user_id).await?;
    Ok(Json(zotero::collections(&query.library, &account.api_key).await?))
}

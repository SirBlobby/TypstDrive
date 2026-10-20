use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};

use crate::admin::users::require_admin;
use crate::db::sql;
use crate::AppState;

const MAX_VERSIONS_KEY: &str = "storage.max_versions_per_document";
const HIGHEST_ALLOWED_LIMIT: i64 = 100_000;

type ApiError = (StatusCode, String);

#[derive(Serialize)]
pub struct VersionLimitResponse {
    pub max_versions: i64,
    pub removed_versions: u64,
}

#[derive(Deserialize)]
pub struct VersionLimitRequest {
    pub max_versions: i64,
}

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

pub async fn max_versions(state: &AppState) -> i64 {
    crate::admin::site_settings::load_settings(state, "storage.")
        .await
        .get(MAX_VERSIONS_KEY)
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

pub async fn prune_document(state: &AppState, document_id: &str, limit: i64) -> Result<u64, sqlx::Error> {
    if limit <= 0 {
        return Ok(0);
    }
    let version_ids = sqlx::query_as::<_, (String,)>(sql(
        "SELECT id FROM document_versions WHERE document_id = $1 ORDER BY created_at DESC, id DESC",
    ))
    .bind(document_id)
    .fetch_all(&state.db)
    .await?;

    let mut removed = 0;
    for (version_id,) in version_ids.into_iter().skip(limit as usize) {
        removed += sqlx::query(sql("DELETE FROM document_versions WHERE id = $1"))
            .bind(&version_id)
            .execute(&state.db)
            .await?
            .rows_affected();
    }
    Ok(removed)
}

pub async fn enforce_limit(state: &AppState, document_id: &str) -> Result<u64, sqlx::Error> {
    let limit = max_versions(state).await;
    prune_document(state, document_id, limit).await
}

async fn prune_all_documents(state: &AppState, limit: i64) -> Result<u64, sqlx::Error> {
    if limit <= 0 {
        return Ok(0);
    }
    let documents = sqlx::query_as::<_, (String,)>(sql(
        "SELECT document_id FROM document_versions GROUP BY document_id HAVING COUNT(*) > $1",
    ))
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    let mut removed = 0;
    for (document_id,) in documents {
        removed += prune_document(state, &document_id, limit).await?;
    }
    Ok(removed)
}

pub async fn get_version_limit(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<VersionLimitResponse>, ApiError> {
    require_admin(&state, &jar).await?;
    Ok(Json(VersionLimitResponse { max_versions: max_versions(&state).await, removed_versions: 0 }))
}

pub async fn update_version_limit(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: crate::accounts::client_info::ClientInfo,
    Json(payload): Json<VersionLimitRequest>,
) -> Result<Json<VersionLimitResponse>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    if !(0..=HIGHEST_ALLOWED_LIMIT).contains(&payload.max_versions) {
        return Err((StatusCode::BAD_REQUEST, format!("The limit must be between 0 and {HIGHEST_ALLOWED_LIMIT}")));
    }

    let value = payload.max_versions.to_string();
    crate::admin::site_settings::save_settings(&state, &[(MAX_VERSIONS_KEY, value.clone())])
        .await
        .map_err(internal_error)?;
    crate::admin::audit::record(&state, &admin_id, &client, "settings.storage_max_versions", "", &value).await;

    let removed_versions = prune_all_documents(&state, payload.max_versions).await.map_err(internal_error)?;
    Ok(Json(VersionLimitResponse { max_versions: payload.max_versions, removed_versions }))
}

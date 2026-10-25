use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};

use super::ApiError;
use crate::accounts::client_info::ClientInfo;
use crate::accounts::signed_in::SignedInUser;
use crate::admin::site_settings::{load_settings, save_settings};
use crate::admin::users::require_admin;
use crate::AppState;

const RETENTION_KEY: &str = "trash.retention_days";
pub const DEFAULT_RETENTION_DAYS: i64 = 30;
pub const MAX_RETENTION_DAYS: i64 = 365;

#[derive(Serialize, Deserialize)]
pub struct TrashPolicy {
    pub retention_days: i64,
}

pub async fn retention_days(state: &AppState) -> i64 {
    load_settings(state, "trash.")
        .await
        .get(RETENTION_KEY)
        .and_then(|value| value.parse::<i64>().ok())
        .map(|days| days.clamp(0, MAX_RETENTION_DAYS))
        .unwrap_or(DEFAULT_RETENTION_DAYS)
}

pub async fn own_policy(State(state): State<AppState>, _: SignedInUser) -> Json<TrashPolicy> {
    Json(TrashPolicy { retention_days: retention_days(&state).await })
}

pub async fn get_policy(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<TrashPolicy>, ApiError> {
    require_admin(&state, &jar).await?;
    Ok(Json(TrashPolicy { retention_days: retention_days(&state).await }))
}

pub async fn update_policy(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
    Json(payload): Json<TrashPolicy>,
) -> Result<Json<TrashPolicy>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    if !(0..=MAX_RETENTION_DAYS).contains(&payload.retention_days) {
        return Err((StatusCode::BAD_REQUEST, format!("Keep items for 0 to {MAX_RETENTION_DAYS} days")));
    }
    save_settings(&state, &[(RETENTION_KEY, payload.retention_days.to_string())])
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    let details = format!("trash kept for {} days", payload.retention_days);
    crate::admin::audit::record(&state, &admin_id, &client, "settings.trash", "", &details).await;
    Ok(Json(payload))
}

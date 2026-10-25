use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};

use super::{cache, ApiError};
use crate::accounts::client_info::ClientInfo;
use crate::admin::site_settings::{load_settings, save_settings};
use crate::admin::users::require_admin;
use crate::AppState;

const PERSONAL_KEY: &str = "themes.personal_enabled";

#[derive(Serialize, Deserialize)]
pub struct ThemePolicy {
    pub personal_enabled: bool,
}

pub async fn personal_enabled(state: &AppState) -> bool {
    load_settings(state, "themes.")
        .await
        .get(PERSONAL_KEY)
        .map(|value| value == "true")
        .unwrap_or(true)
}

pub async fn get_policy(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<ThemePolicy>, ApiError> {
    require_admin(&state, &jar).await?;
    let personal_enabled = state.themes.read().await.personal_enabled;
    Ok(Json(ThemePolicy { personal_enabled }))
}

pub async fn update_policy(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
    Json(payload): Json<ThemePolicy>,
) -> Result<Json<ThemePolicy>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    save_settings(&state, &[(PERSONAL_KEY, payload.personal_enabled.to_string())])
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    let details = format!("personal themes {}", payload.personal_enabled);
    crate::admin::audit::record(&state, &admin_id, &client, "settings.themes", "", &details).await;
    cache::changed(&state, None).await;

    Ok(Json(payload))
}

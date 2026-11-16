use std::sync::atomic::{AtomicBool, Ordering};

use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};

use crate::admin::site_settings::{load_settings, save_settings};
use crate::admin::users::require_admin;
use crate::compile::latex::is_latex_path;
use crate::AppState;

pub const TURNED_OFF_MESSAGE: &str = "LaTeX is turned off on this server.";

const ENABLED_KEY: &str = "latex.enabled";

static ENABLED: AtomicBool = AtomicBool::new(true);

type ApiError = (StatusCode, String);

#[derive(Serialize, Deserialize)]
pub struct LatexPolicy {
    pub enabled: bool,
}

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn check_enabled() -> Result<(), ApiError> {
    if is_enabled() {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, TURNED_OFF_MESSAGE.to_string()))
    }
}

pub fn check_entrypoint(entrypoint: &str) -> Result<(), ApiError> {
    if is_latex_path(entrypoint) { check_enabled() } else { Ok(()) }
}

pub async fn reload(state: &AppState) {
    let settings = load_settings(state, "latex.").await;
    let enabled = settings.get(ENABLED_KEY).map(|value| value == "true").unwrap_or(true);
    ENABLED.store(enabled, Ordering::Relaxed);
}

pub async fn own_policy() -> Json<LatexPolicy> {
    Json(LatexPolicy { enabled: is_enabled() })
}

pub async fn get_policy(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<LatexPolicy>, ApiError> {
    require_admin(&state, &jar).await?;
    reload(&state).await;
    Ok(Json(LatexPolicy { enabled: is_enabled() }))
}

pub async fn update_policy(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: crate::accounts::client_info::ClientInfo,
    Json(payload): Json<LatexPolicy>,
) -> Result<Json<LatexPolicy>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    save_settings(&state, &[(ENABLED_KEY, payload.enabled.to_string())])
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    reload(&state).await;

    let summary = if payload.enabled { "LaTeX turned on" } else { "LaTeX turned off" };
    crate::admin::audit::record(&state, &admin_id, &client, "settings.latex", "", summary).await;
    Ok(Json(LatexPolicy { enabled: is_enabled() }))
}

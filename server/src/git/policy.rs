use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};

use super::remote::Provider;
use super::ApiError;
use crate::admin::site_settings::{load_settings, save_settings};
use crate::admin::users::require_admin;
use crate::AppState;

const ENABLED_KEY: &str = "git.enabled";
const GENERIC_HOSTS_KEY: &str = "git.generic_hosts_enabled";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitPolicy {
    pub enabled: bool,
    pub generic_hosts_enabled: bool,
}

impl GitPolicy {
    pub fn check_enabled(&self) -> Result<(), ApiError> {
        if self.enabled {
            Ok(())
        } else {
            Err((StatusCode::FORBIDDEN, "Git sync is turned off on this server".to_string()))
        }
    }

    pub fn check_provider(&self, provider: Provider) -> Result<(), ApiError> {
        self.check_enabled()?;
        if provider == Provider::Generic && !self.generic_hosts_enabled {
            return Err((StatusCode::FORBIDDEN, "Only GitHub and GitLab repositories can be linked on this server".to_string()));
        }
        Ok(())
    }
}

pub async fn load_policy(state: &AppState) -> GitPolicy {
    let settings = load_settings(state, "git.").await;
    let flag = |key: &str| settings.get(key).map(|value| value == "true").unwrap_or(true);
    GitPolicy { enabled: flag(ENABLED_KEY), generic_hosts_enabled: flag(GENERIC_HOSTS_KEY) }
}

pub async fn own_policy(State(state): State<AppState>) -> Json<GitPolicy> {
    Json(load_policy(&state).await)
}

pub async fn get_policy(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<GitPolicy>, ApiError> {
    require_admin(&state, &jar).await?;
    Ok(Json(load_policy(&state).await))
}

pub async fn update_policy(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: crate::accounts::client_info::ClientInfo,
    Json(payload): Json<GitPolicy>,
) -> Result<Json<GitPolicy>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    let entries = [
        (ENABLED_KEY, payload.enabled.to_string()),
        (GENERIC_HOSTS_KEY, payload.generic_hosts_enabled.to_string()),
    ];
    save_settings(&state, &entries)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    crate::admin::audit::record(
        &state,
        &admin_id,
        &client,
        "settings.git",
        "",
        &format!("git sync {}, generic hosts {}", payload.enabled, payload.generic_hosts_enabled),
    )
    .await;

    Ok(Json(load_policy(&state).await))
}

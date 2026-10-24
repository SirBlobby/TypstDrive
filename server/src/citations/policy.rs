use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};

use super::ApiError;
use crate::admin::site_settings::{load_settings, save_settings};
use crate::admin::users::require_admin;
use crate::AppState;

const LOOKUP_KEY: &str = "citations.lookup_enabled";
const ZOTERO_KEY: &str = "citations.zotero_enabled";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationPolicy {
    pub lookup_enabled: bool,
    pub zotero_enabled: bool,
}

impl CitationPolicy {
    pub fn check_lookup(&self) -> Result<(), ApiError> {
        if self.lookup_enabled {
            Ok(())
        } else {
            Err((StatusCode::FORBIDDEN, "DOI and arXiv lookups are turned off on this server".to_string()))
        }
    }

    pub fn check_zotero(&self) -> Result<(), ApiError> {
        if self.zotero_enabled {
            Ok(())
        } else {
            Err((StatusCode::FORBIDDEN, "Zotero sync is turned off on this server".to_string()))
        }
    }
}

pub async fn load_policy(state: &AppState) -> CitationPolicy {
    let settings = load_settings(state, "citations.").await;
    let flag = |key: &str| settings.get(key).map(|value| value == "true").unwrap_or(true);
    CitationPolicy { lookup_enabled: flag(LOOKUP_KEY), zotero_enabled: flag(ZOTERO_KEY) }
}

pub async fn own_policy(State(state): State<AppState>) -> Json<CitationPolicy> {
    Json(load_policy(&state).await)
}

pub async fn get_policy(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<CitationPolicy>, ApiError> {
    require_admin(&state, &jar).await?;
    Ok(Json(load_policy(&state).await))
}

pub async fn update_policy(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: crate::accounts::client_info::ClientInfo,
    Json(payload): Json<CitationPolicy>,
) -> Result<Json<CitationPolicy>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    let entries = [
        (LOOKUP_KEY, payload.lookup_enabled.to_string()),
        (ZOTERO_KEY, payload.zotero_enabled.to_string()),
    ];
    save_settings(&state, &entries)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::admin::audit::record(
        &state,
        &admin_id,
        &client,
        "settings.citations",
        "",
        &format!("lookups {}, zotero {}", payload.lookup_enabled, payload.zotero_enabled),
    )
    .await;

    Ok(Json(load_policy(&state).await))
}

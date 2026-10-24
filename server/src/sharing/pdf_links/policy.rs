use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};

use super::ApiError;
use crate::admin::site_settings::{load_settings, save_settings};
use crate::admin::users::require_admin;
use crate::AppState;

const ENABLED_KEY: &str = "pdf_links.enabled";
const MAX_PER_USER_KEY: &str = "pdf_links.max_per_user";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfLinkPolicy {
    pub enabled: bool,
    pub max_links_per_user: i64,
}

impl PdfLinkPolicy {
    pub fn check_enabled(&self) -> Result<(), ApiError> {
        if self.enabled {
            Ok(())
        } else {
            Err((StatusCode::FORBIDDEN, "Public PDF links are turned off on this instance".to_string()))
        }
    }

    pub fn check_room(&self, existing_links: i64) -> Result<(), ApiError> {
        if self.max_links_per_user > 0 && existing_links >= self.max_links_per_user {
            return Err((
                StatusCode::FORBIDDEN,
                format!("You already have the maximum of {} public PDF links", self.max_links_per_user),
            ));
        }
        Ok(())
    }
}

pub async fn load_policy(state: &AppState) -> PdfLinkPolicy {
    let settings = load_settings(state, "pdf_links.").await;
    PdfLinkPolicy {
        enabled: settings.get(ENABLED_KEY).map(|value| value == "true").unwrap_or(true),
        max_links_per_user: settings
            .get(MAX_PER_USER_KEY)
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(0)
            .max(0),
    }
}

pub async fn get_policy(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<PdfLinkPolicy>, ApiError> {
    require_admin(&state, &jar).await?;
    Ok(Json(load_policy(&state).await))
}

pub async fn update_policy(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: crate::accounts::client_info::ClientInfo,
    Json(payload): Json<PdfLinkPolicy>,
) -> Result<Json<PdfLinkPolicy>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    if payload.max_links_per_user < 0 {
        return Err((StatusCode::BAD_REQUEST, "The limit cannot be negative".to_string()));
    }

    let entries = [
        (ENABLED_KEY, payload.enabled.to_string()),
        (MAX_PER_USER_KEY, payload.max_links_per_user.to_string()),
    ];
    save_settings(&state, &entries)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::admin::audit::record(
        &state,
        &admin_id,
        &client,
        "settings.pdf_links",
        "",
        &format!("enabled {}, max per user {}", payload.enabled, payload.max_links_per_user),
    )
    .await;

    Ok(Json(load_policy(&state).await))
}

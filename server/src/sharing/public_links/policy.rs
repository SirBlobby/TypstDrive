use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};

use super::{ApiError, Format};
use crate::admin::site_settings::{load_settings, save_settings};
use crate::admin::users::require_admin;
use crate::AppState;

const PDF_ENABLED_KEY: &str = "public_links.pdf_enabled";
const SLIDES_ENABLED_KEY: &str = "public_links.slides_enabled";
const SPEAKER_ENABLED_KEY: &str = "public_links.speaker_enabled";
const MAX_PER_USER_KEY: &str = "public_links.max_per_user";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicLinkPolicy {
    pub pdf_enabled: bool,
    pub slides_enabled: bool,
    pub speaker_enabled: bool,
    pub max_links_per_user: i64,
}

impl PublicLinkPolicy {
    pub fn allows(&self, format: Format) -> bool {
        match format {
            Format::Pdf => self.pdf_enabled,
            Format::Slides => self.slides_enabled,
            Format::Speaker => self.speaker_enabled,
        }
    }

    pub fn any_enabled(&self) -> bool {
        self.pdf_enabled || self.slides_enabled || self.speaker_enabled
    }

    pub fn check_enabled(&self) -> Result<(), ApiError> {
        if self.any_enabled() {
            Ok(())
        } else {
            Err((StatusCode::FORBIDDEN, "Public links are turned off on this instance".to_string()))
        }
    }

    pub fn check_format(&self, format: Format) -> Result<(), ApiError> {
        if self.allows(format) {
            Ok(())
        } else {
            Err((StatusCode::FORBIDDEN, format!("Public {} links are turned off on this instance", format.label())))
        }
    }

    pub fn check_room(&self, existing_links: i64) -> Result<(), ApiError> {
        if self.max_links_per_user > 0 && existing_links >= self.max_links_per_user {
            return Err((
                StatusCode::FORBIDDEN,
                format!("You already have the maximum of {} public links", self.max_links_per_user),
            ));
        }
        Ok(())
    }
}

fn is_on(settings: &std::collections::HashMap<String, String>, key: &str) -> bool {
    settings.get(key).map(|value| value == "true").unwrap_or(true)
}

pub async fn load_policy(state: &AppState) -> PublicLinkPolicy {
    let settings = load_settings(state, "public_links.").await;
    PublicLinkPolicy {
        pdf_enabled: is_on(&settings, PDF_ENABLED_KEY),
        slides_enabled: is_on(&settings, SLIDES_ENABLED_KEY),
        speaker_enabled: is_on(&settings, SPEAKER_ENABLED_KEY),
        max_links_per_user: settings
            .get(MAX_PER_USER_KEY)
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(0)
            .max(0),
    }
}

pub async fn get_policy(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<PublicLinkPolicy>, ApiError> {
    require_admin(&state, &jar).await?;
    Ok(Json(load_policy(&state).await))
}

pub async fn update_policy(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: crate::accounts::client_info::ClientInfo,
    Json(payload): Json<PublicLinkPolicy>,
) -> Result<Json<PublicLinkPolicy>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    if payload.max_links_per_user < 0 {
        return Err((StatusCode::BAD_REQUEST, "The limit cannot be negative".to_string()));
    }

    let entries = [
        (PDF_ENABLED_KEY, payload.pdf_enabled.to_string()),
        (SLIDES_ENABLED_KEY, payload.slides_enabled.to_string()),
        (SPEAKER_ENABLED_KEY, payload.speaker_enabled.to_string()),
        (MAX_PER_USER_KEY, payload.max_links_per_user.to_string()),
    ];
    save_settings(&state, &entries).await.map_err(crate::server_errors::hidden)?;

    crate::admin::audit::record(
        &state,
        &admin_id,
        &client,
        "settings.public_links",
        "",
        &format!(
            "PDF {}, slides {}, speaker view {}, max per user {}",
            payload.pdf_enabled, payload.slides_enabled, payload.speaker_enabled, payload.max_links_per_user
        ),
    )
    .await;

    Ok(Json(load_policy(&state).await))
}

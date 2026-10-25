mod logo;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};

use crate::accounts::client_info::ClientInfo;
use crate::admin::site_settings::{load_settings, save_settings};
use crate::admin::users::require_admin;
use crate::AppState;

pub use logo::{get_logo, logo_updated_at, remove_logo, upload_logo};

const SITE_NAME_KEY: &str = "branding.site_name";
const TAGLINE_KEY: &str = "branding.tagline";
const ACCENT_COLOR_KEY: &str = "branding.accent_color";
const LOGIN_MESSAGE_KEY: &str = "branding.login_message";

const DEFAULT_SITE_NAME: &str = "TypstDrive";
const DEFAULT_TAGLINE: &str = "A collaborative Typst editor and document manager.";

const MAX_SITE_NAME_LENGTH: usize = 40;
const MAX_TAGLINE_LENGTH: usize = 120;
const MAX_LOGIN_MESSAGE_LENGTH: usize = 300;

type ApiError = (StatusCode, String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrandingSettings {
    pub site_name: String,
    pub tagline: String,
    pub accent_color: String,
    pub login_message: String,
}

impl Default for BrandingSettings {
    fn default() -> Self {
        Self {
            site_name: DEFAULT_SITE_NAME.to_string(),
            tagline: DEFAULT_TAGLINE.to_string(),
            accent_color: String::new(),
            login_message: String::new(),
        }
    }
}

#[derive(Serialize)]
pub struct Branding {
    #[serde(flatten)]
    pub settings: BrandingSettings,
    pub logo_updated_at: Option<String>,
    pub customized: bool,
}

fn is_hex_color(value: &str) -> bool {
    value.len() == 7 && value.starts_with('#') && value[1..].chars().all(|character| character.is_ascii_hexdigit())
}

fn too_long(name: &str, value: &str, maximum: usize) -> Result<(), ApiError> {
    if value.chars().count() > maximum {
        Err((StatusCode::BAD_REQUEST, format!("{name} must be {maximum} characters or fewer")))
    } else {
        Ok(())
    }
}

impl BrandingSettings {
    fn normalized(&self) -> Result<Self, ApiError> {
        let settings = Self {
            site_name: self.site_name.trim().to_string(),
            tagline: self.tagline.trim().to_string(),
            accent_color: self.accent_color.trim().to_lowercase(),
            login_message: self.login_message.trim().to_string(),
        };

        if settings.site_name.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "Site name cannot be empty".to_string()));
        }
        too_long("Site name", &settings.site_name, MAX_SITE_NAME_LENGTH)?;
        too_long("Tagline", &settings.tagline, MAX_TAGLINE_LENGTH)?;
        too_long("Sign-in message", &settings.login_message, MAX_LOGIN_MESSAGE_LENGTH)?;
        if !settings.accent_color.is_empty() && !is_hex_color(&settings.accent_color) {
            return Err((StatusCode::BAD_REQUEST, "Accent color must look like #1e66f5".to_string()));
        }
        Ok(settings)
    }
}

pub async fn load(state: &AppState) -> BrandingSettings {
    let settings = load_settings(state, "branding.").await;
    let defaults = BrandingSettings::default();
    let text = |key: &str, fallback: String| settings.get(key).cloned().unwrap_or(fallback);

    BrandingSettings {
        site_name: text(SITE_NAME_KEY, defaults.site_name),
        tagline: text(TAGLINE_KEY, defaults.tagline),
        accent_color: text(ACCENT_COLOR_KEY, defaults.accent_color),
        login_message: text(LOGIN_MESSAGE_KEY, defaults.login_message),
    }
}

async fn current_branding(state: &AppState) -> Branding {
    let settings = load(state).await;
    let logo_updated_at = logo::logo_updated_at(state).await;
    let customized = settings != BrandingSettings::default() || logo_updated_at.is_some();
    Branding { settings, logo_updated_at, customized }
}

pub async fn public_branding(State(state): State<AppState>) -> Json<Branding> {
    Json(current_branding(&state).await)
}

pub async fn update_branding(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
    Json(payload): Json<BrandingSettings>,
) -> Result<Json<Branding>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    let settings = payload.normalized()?;

    let entries = [
        (SITE_NAME_KEY, settings.site_name.clone()),
        (TAGLINE_KEY, settings.tagline.clone()),
        (ACCENT_COLOR_KEY, settings.accent_color.clone()),
        (LOGIN_MESSAGE_KEY, settings.login_message.clone()),
    ];
    save_settings(&state, &entries)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    let accent = if settings.accent_color.is_empty() { "theme" } else { settings.accent_color.as_str() };
    let details = format!("name \"{}\", accent {accent}", settings.site_name);
    crate::admin::audit::record(&state, &admin_id, &client, "settings.branding", "", &details).await;

    Ok(Json(current_branding(&state).await))
}

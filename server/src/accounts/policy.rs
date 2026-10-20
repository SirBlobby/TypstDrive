use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};

use crate::admin::users::require_admin;
use crate::admin::site_settings::{load_settings, save_settings};
use crate::AppState;

const REGISTRATION_KEY: &str = "accounts.registration_enabled";
const DOMAINS_KEY: &str = "accounts.allowed_email_domains";
const USERNAME_CHANGE_KEY: &str = "accounts.allow_username_change";
const EMAIL_CHANGE_KEY: &str = "accounts.allow_email_change";

type ApiError = (StatusCode, String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountPolicy {
    pub registration_enabled: bool,
    pub allowed_email_domains: Vec<String>,
    pub allow_username_change: bool,
    pub allow_email_change: bool,
}

impl AccountPolicy {
    pub fn check_email_domain(&self, email: &str) -> Result<(), ApiError> {
        if self.allowed_email_domains.is_empty() {
            return Ok(());
        }
        let domain = email.rsplit_once('@').map(|(_, domain)| domain.trim().to_lowercase()).unwrap_or_default();
        if self.allowed_email_domains.iter().any(|allowed| *allowed == domain) {
            Ok(())
        } else {
            Err((
                StatusCode::FORBIDDEN,
                format!("Only email addresses from {} can be used", self.allowed_email_domains.join(", ")),
            ))
        }
    }
}

fn normalize_domains(domains: &[String]) -> Vec<String> {
    let mut normalized: Vec<String> = domains
        .iter()
        .map(|domain| domain.trim().trim_start_matches('@').to_lowercase())
        .filter(|domain| !domain.is_empty())
        .collect();
    normalized.sort();
    normalized.dedup();
    normalized
}

pub async fn load_policy(state: &AppState) -> AccountPolicy {
    let settings = load_settings(state, "accounts.").await;
    let flag = |key: &str, fallback: bool| settings.get(key).map(|value| value == "true").unwrap_or(fallback);

    AccountPolicy {
        registration_enabled: flag(REGISTRATION_KEY, state.registration_enabled),
        allowed_email_domains: settings
            .get(DOMAINS_KEY)
            .map(|value| normalize_domains(&value.split(',').map(str::to_string).collect::<Vec<_>>()))
            .unwrap_or_default(),
        allow_username_change: flag(USERNAME_CHANGE_KEY, true),
        allow_email_change: flag(EMAIL_CHANGE_KEY, true),
    }
}

pub async fn public_policy(State(state): State<AppState>) -> Json<AccountPolicy> {
    Json(load_policy(&state).await)
}

pub async fn get_policy(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<AccountPolicy>, ApiError> {
    require_admin(&state, &jar).await?;
    Ok(Json(load_policy(&state).await))
}

pub async fn update_policy(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<AccountPolicy>,
) -> Result<Json<AccountPolicy>, ApiError> {
    require_admin(&state, &jar).await?;

    let domains = normalize_domains(&payload.allowed_email_domains);
    if let Some(invalid) = domains.iter().find(|domain| !domain.contains('.') || domain.contains(char::is_whitespace)) {
        return Err((StatusCode::BAD_REQUEST, format!("'{invalid}' is not a valid email domain")));
    }

    let entries = [
        (REGISTRATION_KEY, payload.registration_enabled.to_string()),
        (DOMAINS_KEY, domains.join(",")),
        (USERNAME_CHANGE_KEY, payload.allow_username_change.to_string()),
        (EMAIL_CHANGE_KEY, payload.allow_email_change.to_string()),
    ];
    save_settings(&state, &entries)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(load_policy(&state).await))
}

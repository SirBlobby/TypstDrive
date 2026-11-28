use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::accounts::login_guard::LockoutRules;
use crate::admin::site_settings::{load_settings, save_settings};
use crate::AppState;

const PASSWORD_LOGIN_KEY: &str = "auth.password_login_enabled";
const EMAIL_CODE_LOGIN_KEY: &str = "auth.email_code_login_enabled";
const REQUIRE_VERIFICATION_KEY: &str = "auth.require_email_verification";
const AUTO_CREATE_KEY: &str = "auth.auto_create_accounts";
const DISABLED_PROVIDERS_KEY: &str = "auth.disabled_providers";
const MAX_FAILED_LOGINS_KEY: &str = "auth.max_failed_logins";
const LOCKOUT_MINUTES_KEY: &str = "auth.lockout_minutes";
const SESSION_DAYS_KEY: &str = "auth.session_days";
const MIN_PASSWORD_LENGTH_KEY: &str = "auth.min_password_length";
const TEMPORARY_PASSWORD_HOURS_KEY: &str = "auth.temporary_password_hours";
const AUTHENTICATOR_ALLOWED_KEY: &str = "auth.authenticator_app_allowed";
const EMAIL_TWO_FACTOR_ALLOWED_KEY: &str = "auth.email_two_factor_allowed";

const MAX_FAILED_LOGINS_LIMIT: i64 = 100;
const MAX_LOCKOUT_MINUTES: i64 = 24 * 60;
const MAX_SESSION_DAYS: i64 = 365;
const MAX_PASSWORD_LENGTH_SETTING: i64 = 128;
const MAX_TEMPORARY_PASSWORD_HOURS: i64 = 30 * 24;
const DEFAULT_TEMPORARY_PASSWORD_HOURS: i64 = 72;

type ApiError = (StatusCode, String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSettings {
    pub password_login_enabled: bool,
    #[serde(default = "enabled_by_default")]
    pub email_code_login_enabled: bool,
    pub require_email_verification: bool,
    pub auto_create_accounts: bool,
    pub disabled_providers: Vec<String>,
    pub max_failed_logins: i64,
    pub lockout_minutes: i64,
    pub session_days: i64,
    #[serde(default = "default_min_password_length")]
    pub min_password_length: i64,
    #[serde(default = "default_temporary_password_hours")]
    pub temporary_password_hours: i64,
    #[serde(default = "enabled_by_default")]
    pub authenticator_app_allowed: bool,
    #[serde(default = "enabled_by_default")]
    pub email_two_factor_allowed: bool,
}

fn enabled_by_default() -> bool {
    true
}

fn default_min_password_length() -> i64 {
    crate::accounts::passwords::DEFAULT_MIN_PASSWORD_LENGTH
}

fn default_temporary_password_hours() -> i64 {
    DEFAULT_TEMPORARY_PASSWORD_HOURS
}

impl Default for AuthSettings {
    fn default() -> Self {
        Self {
            password_login_enabled: true,
            email_code_login_enabled: true,
            require_email_verification: false,
            auto_create_accounts: false,
            disabled_providers: Vec::new(),
            max_failed_logins: 5,
            lockout_minutes: 15,
            session_days: 30,
            min_password_length: default_min_password_length(),
            temporary_password_hours: DEFAULT_TEMPORARY_PASSWORD_HOURS,
            authenticator_app_allowed: true,
            email_two_factor_allowed: true,
        }
    }
}

impl AuthSettings {
    pub fn lockout_rules(&self) -> LockoutRules {
        LockoutRules {
            max_failures: self.max_failed_logins.max(0) as u32,
            lockout: Duration::from_secs(self.lockout_minutes.max(1) as u64 * 60),
        }
    }

    pub fn provider_enabled(&self, provider_id: &str) -> bool {
        !self.disabled_providers.iter().any(|disabled| disabled == provider_id)
    }

    pub fn validate(&self) -> Result<(), ApiError> {
        let in_range = |name: &str, value: i64, minimum: i64, maximum: i64| {
            if value < minimum || value > maximum {
                Err((StatusCode::BAD_REQUEST, format!("{name} must be between {minimum} and {maximum}")))
            } else {
                Ok(())
            }
        };
        in_range("Failed sign-ins before lockout", self.max_failed_logins, 0, MAX_FAILED_LOGINS_LIMIT)?;
        in_range("Lockout length", self.lockout_minutes, 1, MAX_LOCKOUT_MINUTES)?;
        in_range("Session length", self.session_days, 1, MAX_SESSION_DAYS)?;
        in_range("Minimum password length", self.min_password_length, default_min_password_length(), MAX_PASSWORD_LENGTH_SETTING)?;
        in_range("Temporary password lifetime", self.temporary_password_hours, 1, MAX_TEMPORARY_PASSWORD_HOURS)?;
        if !self.authenticator_app_allowed && !self.email_two_factor_allowed {
            return Err((
                StatusCode::BAD_REQUEST,
                "Allow at least one two-factor method. Administrators need two-factor authentication to manage users.".to_string(),
            ));
        }
        Ok(())
    }
}

pub async fn load(state: &AppState) -> AuthSettings {
    let settings = load_settings(state, "auth.").await;
    let defaults = AuthSettings::default();
    let flag = |key: &str, fallback: bool| settings.get(key).map(|value| value == "true").unwrap_or(fallback);
    let number = |key: &str, fallback: i64| settings.get(key).and_then(|value| value.parse().ok()).unwrap_or(fallback);

    AuthSettings {
        password_login_enabled: flag(PASSWORD_LOGIN_KEY, defaults.password_login_enabled),
        email_code_login_enabled: flag(EMAIL_CODE_LOGIN_KEY, defaults.email_code_login_enabled),
        require_email_verification: flag(REQUIRE_VERIFICATION_KEY, defaults.require_email_verification),
        auto_create_accounts: flag(AUTO_CREATE_KEY, defaults.auto_create_accounts),
        disabled_providers: settings
            .get(DISABLED_PROVIDERS_KEY)
            .map(|value| value.split(',').map(str::trim).filter(|id| !id.is_empty()).map(str::to_string).collect())
            .unwrap_or_default(),
        max_failed_logins: number(MAX_FAILED_LOGINS_KEY, defaults.max_failed_logins),
        lockout_minutes: number(LOCKOUT_MINUTES_KEY, defaults.lockout_minutes),
        session_days: number(SESSION_DAYS_KEY, defaults.session_days),
        min_password_length: number(MIN_PASSWORD_LENGTH_KEY, defaults.min_password_length),
        temporary_password_hours: number(TEMPORARY_PASSWORD_HOURS_KEY, defaults.temporary_password_hours),
        authenticator_app_allowed: flag(AUTHENTICATOR_ALLOWED_KEY, defaults.authenticator_app_allowed),
        email_two_factor_allowed: flag(EMAIL_TWO_FACTOR_ALLOWED_KEY, defaults.email_two_factor_allowed),
    }
}

pub async fn save(state: &AppState, settings: &AuthSettings) -> Result<(), ApiError> {
    settings.validate()?;
    let entries = [
        (PASSWORD_LOGIN_KEY, settings.password_login_enabled.to_string()),
        (EMAIL_CODE_LOGIN_KEY, settings.email_code_login_enabled.to_string()),
        (REQUIRE_VERIFICATION_KEY, settings.require_email_verification.to_string()),
        (AUTO_CREATE_KEY, settings.auto_create_accounts.to_string()),
        (DISABLED_PROVIDERS_KEY, settings.disabled_providers.join(",")),
        (MAX_FAILED_LOGINS_KEY, settings.max_failed_logins.to_string()),
        (LOCKOUT_MINUTES_KEY, settings.lockout_minutes.to_string()),
        (SESSION_DAYS_KEY, settings.session_days.to_string()),
        (MIN_PASSWORD_LENGTH_KEY, settings.min_password_length.to_string()),
        (TEMPORARY_PASSWORD_HOURS_KEY, settings.temporary_password_hours.to_string()),
        (AUTHENTICATOR_ALLOWED_KEY, settings.authenticator_app_allowed.to_string()),
        (EMAIL_TWO_FACTOR_ALLOWED_KEY, settings.email_two_factor_allowed.to_string()),
    ];
    save_settings(state, &entries)
        .await
        .map_err(crate::server_errors::hidden)
}

pub fn email_links_available(state: &AppState) -> bool {
    state.mailer.is_configured() && crate::site_url::public_url().is_some()
}

pub fn email_code_login_available(state: &AppState, settings: &AuthSettings) -> bool {
    settings.email_code_login_enabled && state.mailer.is_configured()
}

pub fn verification_required(state: &AppState, settings: &AuthSettings) -> bool {
    settings.require_email_verification && email_links_available(state)
}

#[derive(Serialize)]
pub struct ProviderButton {
    pub id: String,
    pub name: String,
    pub kind: &'static str,
}

#[derive(Serialize)]
pub struct AuthOptions {
    pub password_login_enabled: bool,
    pub registration_enabled: bool,
    pub password_reset_available: bool,
    pub email_code_login_available: bool,
    pub email_verification_available: bool,
    pub verification_required: bool,
    pub min_password_length: i64,
    pub providers: Vec<ProviderButton>,
}

pub async fn auth_options(State(state): State<AppState>) -> Json<AuthOptions> {
    let settings = load(&state).await;
    let policy = crate::accounts::policy::load_policy(&state).await;
    let providers = state
        .sso
        .usable_providers()
        .filter(|provider| settings.provider_enabled(&provider.id))
        .map(|provider| ProviderButton {
            id: provider.id.clone(),
            name: provider.name.clone(),
            kind: provider.kind.label(),
        })
        .collect();

    Json(AuthOptions {
        password_login_enabled: settings.password_login_enabled,
        registration_enabled: policy.registration_enabled && settings.password_login_enabled,
        password_reset_available: settings.password_login_enabled && email_links_available(&state),
        email_code_login_available: email_code_login_available(&state, &settings),
        email_verification_available: email_links_available(&state),
        verification_required: verification_required(&state, &settings),
        min_password_length: settings.min_password_length,
        providers,
    })
}

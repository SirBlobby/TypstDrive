use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};

use crate::accounts::auth_settings::{self, AuthSettings};
use crate::accounts::client_info::ClientInfo;
use crate::admin::{audit, users::require_admin};
use crate::email::EmailStatus;
use crate::AppState;

type ApiError = (StatusCode, String);

#[derive(Serialize)]
pub struct ProviderStatus {
    pub id: String,
    pub name: String,
    pub kind: &'static str,
    pub enabled: bool,
    pub callback_url: Option<String>,
}

#[derive(Serialize)]
pub struct AuthenticationOverview {
    pub settings: AuthSettings,
    pub public_url: Option<String>,
    pub email: EmailStatus,
    pub verification_available: bool,
    pub providers: Vec<ProviderStatus>,
    pub provider_problems: Vec<String>,
}

#[derive(Deserialize)]
pub struct TestEmailRequest {
    pub recipient: Option<String>,
}

async fn overview(state: &AppState) -> AuthenticationOverview {
    let settings = auth_settings::load(state).await;
    let providers = state
        .sso
        .configured_providers()
        .map(|provider| ProviderStatus {
            id: provider.id.clone(),
            name: provider.name.clone(),
            kind: provider.kind.label(),
            enabled: settings.provider_enabled(&provider.id),
            callback_url: provider.callback_url(),
        })
        .collect();

    AuthenticationOverview {
        public_url: crate::site_url::public_url(),
        email: state.mailer.status(),
        verification_available: auth_settings::email_links_available(state),
        providers,
        provider_problems: state.sso.problems().to_vec(),
        settings,
    }
}

fn describe(settings: &AuthSettings) -> String {
    format!(
        "password sign-in {}, email codes {}, verification {}, auto-create {}, disabled providers [{}], lockout after {} for {} min, sessions {} days",
        settings.password_login_enabled,
        settings.email_code_login_enabled,
        settings.require_email_verification,
        settings.auto_create_accounts,
        settings.disabled_providers.join(", "),
        settings.max_failed_logins,
        settings.lockout_minutes,
        settings.session_days
    )
}

pub async fn get_authentication(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<AuthenticationOverview>, ApiError> {
    require_admin(&state, &jar).await?;
    Ok(Json(overview(&state).await))
}

pub async fn update_authentication(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
    Json(payload): Json<AuthSettings>,
) -> Result<Json<AuthenticationOverview>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    auth_settings::save(&state, &payload).await?;
    audit::record(&state, &admin_id, &client, "settings.authentication", "", &describe(&payload)).await;
    Ok(Json(overview(&state).await))
}

pub async fn send_test_email(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
    Json(payload): Json<TestEmailRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    let admin = crate::accounts::lookup::user_by_id(&state, &admin_id)
        .await?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    let recipient = payload
        .recipient
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| admin.email.clone());

    state
        .mailer
        .send(&recipient, crate::email::messages::test_message(&crate::email::brand::load(&state).await, &admin.username))
        .await
        .map_err(|error| (StatusCode::BAD_GATEWAY, format!("Sending failed: {error}")))?;

    audit::record(&state, &admin_id, &client, "email.test", &recipient, "").await;
    Ok(Json(serde_json::json!({ "sent_to": recipient })))
}

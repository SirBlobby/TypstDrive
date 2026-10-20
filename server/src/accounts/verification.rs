use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::{CookieJar, SignedCookieJar};
use serde::Deserialize;

use crate::accounts::client_info::ClientInfo;
use crate::accounts::email_code::CODE_ATTEMPT_RULES;
use crate::accounts::email_tokens::{self, TokenPurpose, VERIFY_EMAIL_CODE_MINUTES};
use crate::accounts::{auth, lookup, sessions};
use crate::db::sql;
use crate::models::UserProfile;
use crate::AppState;

const VERIFY_CODE_SCOPE: &str = "verify_email_code";

type ApiError = (StatusCode, String);

#[derive(Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

#[derive(Deserialize)]
pub struct VerifyCodeRequest {
    pub email: String,
    pub code: String,
}

#[derive(Deserialize)]
pub struct ResendVerificationRequest {
    pub email: Option<String>,
}

pub async fn send_verification(state: &AppState, user_id: &str, username: &str, email: &str) -> Result<(), ApiError> {
    if !crate::accounts::auth_settings::email_links_available(state) {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "Email is not configured on this server".to_string()));
    }
    let token = email_tokens::issue(state, user_id, email, TokenPurpose::VerifyEmail)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    let link = crate::site_url::link(&format!("/verify-email?token={token}")).unwrap_or_default();
    let code = email_tokens::issue_code(state, user_id, email, TokenPurpose::VerifyEmailCode)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    state
        .mailer
        .send(email, crate::email::messages::email_verification(username, &link, &code, VERIFY_EMAIL_CODE_MINUTES))
        .await
        .map_err(|error| (StatusCode::BAD_GATEWAY, format!("Could not send the email: {error}")))
}

pub async fn send_verification_quietly(state: &AppState, user_id: &str, username: &str, email: &str) {
    if !crate::accounts::auth_settings::email_links_available(state) {
        return;
    }
    if let Err((_, message)) = send_verification(state, user_id, username, email).await {
        tracing::warn!("Verification email to {email} failed: {message}");
    }
}

pub async fn verify_email(
    State(state): State<AppState>,
    Json(payload): Json<VerifyEmailRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let invalid = (StatusCode::BAD_REQUEST, "This verification link is invalid or has expired".to_string());
    let consumed = email_tokens::consume(&state, &payload.token, TokenPurpose::VerifyEmail).await.ok_or(invalid)?;

    if mark_verified(&state, &consumed.user_id, &consumed.email).await? == 0 {
        return Err((StatusCode::BAD_REQUEST, "This link was sent to an email address the account no longer uses".to_string()));
    }
    Ok(Json(serde_json::json!({ "verified": true, "email": consumed.email })))
}

async fn mark_verified(state: &AppState, user_id: &str, email: &str) -> Result<u64, ApiError> {
    sqlx::query(sql("UPDATE users SET email_verified = 1 WHERE id = $1 AND LOWER(email) = LOWER($2)"))
        .bind(user_id)
        .bind(email)
        .execute(&state.db)
        .await
        .map(|result| result.rows_affected())
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
}

pub async fn verify_with_code(
    State(state): State<AppState>,
    client: ClientInfo,
    jar: CookieJar,
    Json(payload): Json<VerifyCodeRequest>,
) -> Result<(CookieJar, Json<UserProfile>), ApiError> {
    let email = payload.email.trim();
    state.login_guard.ensure_allowed(VERIFY_CODE_SCOPE, email, &client.ip_address)?;

    let invalid = (StatusCode::BAD_REQUEST, "The code is wrong or has expired".to_string());
    let user = lookup::user_by_email(&state, email).await?;
    let consumed = match &user {
        Some(user) => email_tokens::consume_code(&state, &user.id, &payload.code, TokenPurpose::VerifyEmailCode).await,
        None => None,
    };
    let (Some(user), Some(consumed)) = (user, consumed) else {
        state.login_guard.record_failure(VERIFY_CODE_SCOPE, email, &client.ip_address, CODE_ATTEMPT_RULES);
        return Err(invalid);
    };
    if mark_verified(&state, &user.id, &consumed.email).await? == 0 {
        return Err(invalid);
    }

    state.login_guard.record_success(VERIFY_CODE_SCOPE, email);
    let cookie = sessions::start_session(&state, &user.id, &client).await?;
    let user = lookup::user_by_id(&state, &user.id).await?.unwrap_or(user);
    let profile = auth::profile_for(&state, user).await;

    Ok((jar.add(cookie), Json(profile)))
}

pub async fn resend_verification(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<ResendVerificationRequest>,
) -> Result<StatusCode, ApiError> {
    let session_user = jar.get("session_user_id").map(|cookie| cookie.value().to_string());
    let user = match (session_user, payload.email) {
        (Some(user_id), _) => lookup::user_by_id(&state, &user_id).await?,
        (None, Some(email)) => lookup::user_by_email(&state, &email).await?,
        (None, None) => return Err((StatusCode::BAD_REQUEST, "Enter your email address".to_string())),
    };

    let Some(user) = user.filter(|user| user.email_verified == 0) else {
        return Ok(StatusCode::ACCEPTED);
    };
    if email_tokens::recently_issued(&state, &user.id, TokenPurpose::VerifyEmail).await {
        return Ok(StatusCode::ACCEPTED);
    }

    send_verification(&state, &user.id, &user.username, &user.email).await?;
    Ok(StatusCode::ACCEPTED)
}

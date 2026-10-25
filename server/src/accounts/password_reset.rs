use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::accounts::client_info::ClientInfo;
use crate::accounts::email_tokens::{self, TokenPurpose};
use crate::accounts::{auth_settings, lookup, passwords, sessions};
use crate::db::sql;
use crate::AppState;

type ApiError = (StatusCode, String);

const RESET_SCOPE: &str = "reset";

#[derive(Deserialize)]
pub struct ResetRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetConfirmation {
    pub token: String,
    pub password: String,
}

pub async fn request_reset(
    State(state): State<AppState>,
    client: ClientInfo,
    Json(payload): Json<ResetRequest>,
) -> Result<StatusCode, ApiError> {
    let settings = auth_settings::load(&state).await;
    if !settings.password_login_enabled || !auth_settings::email_links_available(&state) {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "Password reset by email is not available on this server".to_string()));
    }
    state.login_guard.ensure_allowed(RESET_SCOPE, &payload.email, &client.ip_address)?;
    state.login_guard.record_failure(RESET_SCOPE, &payload.email, &client.ip_address, settings.lockout_rules());

    let Some(user) = lookup::user_by_email(&state, &payload.email).await? else {
        return Ok(StatusCode::ACCEPTED);
    };
    if email_tokens::recently_issued(&state, &user.id, TokenPurpose::ResetPassword).await {
        return Ok(StatusCode::ACCEPTED);
    }

    let token = email_tokens::issue(&state, &user.id, &user.email, TokenPurpose::ResetPassword)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    let link = crate::site_url::link(&format!("/reset-password?token={token}")).unwrap_or_default();

    if let Err(error) = state.mailer.send(&user.email, crate::email::messages::password_reset(&crate::email::brand::load(&state).await, &user.username, &link)).await {
        tracing::warn!("Password reset email to {} failed: {error}", user.email);
    }
    Ok(StatusCode::ACCEPTED)
}

pub async fn confirm_reset(
    State(state): State<AppState>,
    Json(payload): Json<ResetConfirmation>,
) -> Result<StatusCode, ApiError> {
    passwords::check_new_password(&payload.password)?;

    let invalid = (StatusCode::BAD_REQUEST, "This reset link is invalid or has expired".to_string());
    let consumed = email_tokens::consume(&state, &payload.token, TokenPurpose::ResetPassword).await.ok_or(invalid)?;

    let password_hash = passwords::hash_password(&payload.password).await.map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))?;
    sqlx::query(sql(
        "UPDATE users SET password_hash = $1, email_verified = CASE WHEN LOWER(email) = LOWER($2) THEN 1 ELSE email_verified END WHERE id = $3",
    ))
    .bind(&password_hash)
    .bind(&consumed.email)
    .bind(&consumed.user_id)
    .execute(&state.db)
    .await
    .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    sessions::end_all_sessions(&state, &consumed.user_id, None).await?;
    crate::sharing::access::revoke_user_sessions(&state, &consumed.user_id).await;
    state.login_guard.record_success("login", &consumed.email);
    state.login_guard.record_success(RESET_SCOPE, &consumed.email);

    Ok(StatusCode::OK)
}

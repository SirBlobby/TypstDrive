use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;

use crate::accounts::client_info::ClientInfo;
use crate::accounts::email_tokens::{self, TokenPurpose, SIGN_IN_CODE_MINUTES};
use crate::accounts::login_guard::LockoutRules;
use crate::accounts::{auth, auth_settings, lookup, sessions};
use crate::db::sql;
use crate::models::UserProfile;
use crate::AppState;

type ApiError = (StatusCode, String);

const REQUEST_SCOPE: &str = "email_code_request";
const VERIFY_SCOPE: &str = "email_code";

pub const CODE_ATTEMPT_RULES: LockoutRules = LockoutRules {
    max_failures: 5,
    lockout: Duration::from_secs(15 * 60),
};

const REQUEST_RULES: LockoutRules = LockoutRules {
    max_failures: 5,
    lockout: Duration::from_secs(15 * 60),
};

#[derive(Deserialize)]
pub struct CodeRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct CodeSignIn {
    pub email: String,
    pub code: String,
}

async fn ensure_available(state: &AppState) -> Result<(), ApiError> {
    let settings = auth_settings::load(state).await;
    if auth_settings::email_code_login_available(state, &settings) {
        Ok(())
    } else {
        Err((StatusCode::SERVICE_UNAVAILABLE, "Sign-in with an email code is not available on this server".to_string()))
    }
}

pub async fn request_code(
    State(state): State<AppState>,
    client: ClientInfo,
    Json(payload): Json<CodeRequest>,
) -> Result<StatusCode, ApiError> {
    ensure_available(&state).await?;
    let email = payload.email.trim();
    state.login_guard.ensure_allowed(REQUEST_SCOPE, email, &client.ip_address)?;
    state.login_guard.record_failure(REQUEST_SCOPE, email, &client.ip_address, REQUEST_RULES);

    let Some(user) = lookup::user_by_email(&state, email).await? else {
        return Ok(StatusCode::ACCEPTED);
    };
    if email_tokens::recently_issued(&state, &user.id, TokenPurpose::SignInCode).await {
        return Ok(StatusCode::ACCEPTED);
    }

    let code = email_tokens::issue_code(&state, &user.id, &user.email, TokenPurpose::SignInCode)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    let message = crate::email::messages::sign_in_code(&user.username, &code, SIGN_IN_CODE_MINUTES);
    if let Err(error) = state.mailer.send(&user.email, message).await {
        tracing::warn!("Sign-in code email to {} failed: {error}", user.email);
    }
    Ok(StatusCode::ACCEPTED)
}

pub async fn sign_in_with_code(
    State(state): State<AppState>,
    client: ClientInfo,
    jar: CookieJar,
    Json(payload): Json<CodeSignIn>,
) -> Result<(CookieJar, Json<UserProfile>), ApiError> {
    ensure_available(&state).await?;
    let email = payload.email.trim();
    state.login_guard.ensure_allowed(VERIFY_SCOPE, email, &client.ip_address)?;

    let invalid = (StatusCode::UNAUTHORIZED, "The code is wrong or has expired".to_string());
    let user = lookup::user_by_email(&state, email).await?;
    let consumed = match &user {
        Some(user) => email_tokens::consume_code(&state, &user.id, &payload.code, TokenPurpose::SignInCode).await,
        None => None,
    };
    let (Some(user), Some(consumed)) = (user, consumed) else {
        state.login_guard.record_failure(VERIFY_SCOPE, email, &client.ip_address, CODE_ATTEMPT_RULES);
        return Err(invalid);
    };
    if !consumed.email.eq_ignore_ascii_case(&user.email) {
        return Err(invalid);
    }

    if user.email_verified == 0 {
        sqlx::query(sql("UPDATE users SET email_verified = 1 WHERE id = $1"))
            .bind(&user.id)
            .execute(&state.db)
            .await
            .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    }

    state.login_guard.record_success(VERIFY_SCOPE, email);
    state.login_guard.record_success(REQUEST_SCOPE, email);
    let cookie = sessions::start_session(&state, &user.id, &client).await?;
    let user = lookup::user_by_id(&state, &user.id).await?.unwrap_or(user);
    let profile = auth::profile_for(&state, user).await;

    Ok((jar.add(cookie), Json(profile)))
}

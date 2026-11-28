use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use super::{backup_codes, totp, Method};
use crate::accounts::client_info::ClientInfo;
use crate::accounts::email_code::CODE_ATTEMPT_RULES;
use crate::accounts::email_tokens::{self, TokenPurpose};
use crate::accounts::{auth_settings, lookup};
use crate::accounts::random_text::grouped;
use crate::accounts::sessions::CurrentSession;
use crate::db::sql;
use crate::models::User;
use crate::AppState;

type ApiError = (StatusCode, String);

const ATTEMPT_SCOPE: &str = "two_factor_settings";
const SECRET_GROUP_LENGTH: usize = 4;

#[derive(Serialize)]
pub struct TwoFactorStatus {
    method: Option<Method>,
    backup_codes_left: i64,
    authenticator_available: bool,
    email_available: bool,
    authenticator_allowed: bool,
    email_allowed: bool,
}

#[derive(Serialize)]
pub struct AuthenticatorSetup {
    secret: String,
    uri: String,
    qr_size: usize,
    qr_modules: String,
}

#[derive(Deserialize)]
pub struct CodeInput {
    code: String,
}

#[derive(Serialize)]
pub struct BackupCodes {
    backup_codes: Vec<String>,
}

async fn signed_in_user(state: &AppState, user_id: &str) -> Result<User, ApiError> {
    lookup::user_by_id(state, user_id)
        .await?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))
}

fn email_available_for(state: &AppState, user: &User) -> bool {
    state.mailer.is_configured() && user.email_verified != 0
}

async fn ensure_not_set_up(state: &AppState, user_id: &str) -> Result<(), ApiError> {
    match super::method_of(state, user_id).await {
        Some(_) => Err((StatusCode::CONFLICT, "Two-factor authentication is already on. Turn it off first to switch methods.".to_string())),
        None => Ok(()),
    }
}

async fn ensure_allowed(state: &AppState, method: Method) -> Result<(), ApiError> {
    let settings = auth_settings::load(state).await;
    let allowed = match method {
        Method::Totp => settings.authenticator_app_allowed,
        Method::Email => settings.email_two_factor_allowed,
    };
    if allowed {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "An administrator turned off this two-factor method".to_string()))
    }
}

fn wrong_code() -> ApiError {
    (StatusCode::BAD_REQUEST, "The code is wrong or has expired".to_string())
}

async fn check_attempt<F>(state: &AppState, user_id: &str, client: &ClientInfo, accepted: F) -> Result<(), ApiError>
where
    F: std::future::Future<Output = bool>,
{
    state.login_guard.ensure_allowed(ATTEMPT_SCOPE, user_id, &client.ip_address)?;
    if accepted.await {
        state.login_guard.record_success(ATTEMPT_SCOPE, user_id);
        Ok(())
    } else {
        state.login_guard.record_failure(ATTEMPT_SCOPE, user_id, &client.ip_address, CODE_ATTEMPT_RULES);
        Err(wrong_code())
    }
}

async fn enable(state: &AppState, user_id: &str, method: Method, last_step: i64) -> Result<Json<BackupCodes>, ApiError> {
    let updated = sqlx::query(sql(
        "UPDATE users SET two_factor_method = $1, totp_secret = CASE WHEN $1 = 'totp' THEN totp_setup_secret ELSE NULL END, \
         totp_setup_secret = NULL, totp_last_step = $2 WHERE id = $3 AND two_factor_method IS NULL",
    ))
    .bind(method.as_str())
    .bind(last_step)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(crate::server_errors::hidden)?;
    if updated.rows_affected() == 0 {
        return Err((StatusCode::CONFLICT, "Two-factor authentication is already on".to_string()));
    }
    let backup_codes = backup_codes::replace(state, user_id).await?;
    Ok(Json(BackupCodes { backup_codes }))
}

pub async fn status(State(state): State<AppState>, CurrentSession(session): CurrentSession) -> Result<Json<TwoFactorStatus>, ApiError> {
    let user = signed_in_user(&state, &session.user_id).await?;
    let settings = auth_settings::load(&state).await;
    Ok(Json(TwoFactorStatus {
        method: super::method_of(&state, &user.id).await,
        backup_codes_left: backup_codes::remaining(&state, &user.id).await,
        authenticator_available: super::authenticator_available(),
        email_available: email_available_for(&state, &user),
        authenticator_allowed: settings.authenticator_app_allowed,
        email_allowed: settings.email_two_factor_allowed,
    }))
}

pub async fn start_authenticator(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
) -> Result<Json<AuthenticatorSetup>, ApiError> {
    ensure_not_set_up(&state, &session.user_id).await?;
    ensure_allowed(&state, Method::Totp).await?;
    if !super::authenticator_available() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Authenticator apps need COOKIE_SECRET to be set on the server. Ask your administrator.".to_string(),
        ));
    }
    let user = signed_in_user(&state, &session.user_id).await?;
    let secret = totp::generate_secret();
    sqlx::query(sql("UPDATE users SET totp_setup_secret = $1 WHERE id = $2"))
        .bind(super::seal_secret(&state, &secret))
        .bind(&user.id)
        .execute(&state.db)
        .await
        .map_err(crate::server_errors::hidden)?;

    let site_name = crate::email::brand::load(&state).await.site_name;
    let uri = totp::setup_uri(&site_name, &user.email, &secret);
    let qr = totp::qr_code(&uri).ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Could not create the QR code".to_string()))?;
    Ok(Json(AuthenticatorSetup { secret: grouped(&secret, SECRET_GROUP_LENGTH), uri, qr_size: qr.size, qr_modules: qr.modules }))
}

pub async fn confirm_authenticator(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    client: ClientInfo,
    Json(payload): Json<CodeInput>,
) -> Result<Json<BackupCodes>, ApiError> {
    ensure_not_set_up(&state, &session.user_id).await?;
    ensure_allowed(&state, Method::Totp).await?;
    let sealed = sqlx::query_as::<_, (Option<String>,)>(sql("SELECT totp_setup_secret FROM users WHERE id = $1"))
        .bind(&session.user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(crate::server_errors::hidden)?
        .and_then(|(sealed,)| sealed)
        .ok_or((StatusCode::BAD_REQUEST, "Start the authenticator setup again".to_string()))?;
    let secret = super::open_secret(&state, &sealed).ok_or((StatusCode::BAD_REQUEST, "Start the authenticator setup again".to_string()))?;

    let mut matched_step = 0;
    check_attempt(&state, &session.user_id, &client, async {
        matched_step = totp::matching_step(&secret, &payload.code, 0).unwrap_or(0);
        matched_step > 0
    })
    .await?;
    enable(&state, &session.user_id, Method::Totp, matched_step).await
}

pub async fn send_email_code(State(state): State<AppState>, CurrentSession(session): CurrentSession) -> Result<StatusCode, ApiError> {
    let user = signed_in_user(&state, &session.user_id).await?;
    match super::method_of(&state, &user.id).await {
        Some(Method::Totp) => return Err((StatusCode::BAD_REQUEST, "Your account uses an authenticator app".to_string())),
        Some(Method::Email) if state.mailer.is_configured() => {}
        None if email_available_for(&state, &user) => ensure_allowed(&state, Method::Email).await?,
        _ => return Err((StatusCode::SERVICE_UNAVAILABLE, "Email codes need a verified email address and email set up on the server".to_string())),
    }
    super::send_email_code(&state, &user.id, &user.username, &user.email).await?;
    Ok(StatusCode::ACCEPTED)
}

pub async fn confirm_email(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    client: ClientInfo,
    Json(payload): Json<CodeInput>,
) -> Result<Json<BackupCodes>, ApiError> {
    ensure_not_set_up(&state, &session.user_id).await?;
    ensure_allowed(&state, Method::Email).await?;
    check_attempt(&state, &session.user_id, &client, async {
        email_tokens::consume_code(&state, &session.user_id, &payload.code, TokenPurpose::TwoFactorCode).await.is_some()
    })
    .await?;
    enable(&state, &session.user_id, Method::Email, 0).await
}

pub async fn regenerate_backup_codes(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    client: ClientInfo,
    Json(payload): Json<CodeInput>,
) -> Result<Json<BackupCodes>, ApiError> {
    check_attempt(&state, &session.user_id, &client, super::accepts_code(&state, &session.user_id, &payload.code)).await?;
    let backup_codes = backup_codes::replace(&state, &session.user_id).await?;
    Ok(Json(BackupCodes { backup_codes }))
}

pub async fn turn_off(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    client: ClientInfo,
    Json(payload): Json<CodeInput>,
) -> Result<StatusCode, ApiError> {
    check_attempt(&state, &session.user_id, &client, super::accepts_code(&state, &session.user_id, &payload.code)).await?;
    super::turn_off(&state, &session.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

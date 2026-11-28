pub mod account;
pub mod backup_codes;
pub mod totp;

use axum::http::StatusCode;
use serde::Serialize;

use crate::accounts::email_tokens::{self, TokenPurpose, SIGN_IN_CODE_MINUTES};
use crate::db::sql;
use crate::AppState;

type ApiError = (StatusCode, String);

const SEALED_NAME: &str = "totp_secret";
const PERSISTENT_KEY_BYTES: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    Totp,
    Email,
}

impl Method {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Totp => "totp",
            Self::Email => "email",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "totp" => Some(Self::Totp),
            "email" => Some(Self::Email),
            _ => None,
        }
    }
}

pub fn authenticator_available() -> bool {
    std::env::var("COOKIE_SECRET").is_ok_and(|secret| secret.len() >= PERSISTENT_KEY_BYTES)
}

pub fn seal_secret(state: &AppState, secret: &str) -> String {
    crate::sealed_secrets::seal(&state.key, SEALED_NAME, secret)
}

pub fn open_secret(state: &AppState, sealed: &str) -> Option<String> {
    crate::sealed_secrets::open(&state.key, SEALED_NAME, sealed)
}

pub async fn method_of(state: &AppState, user_id: &str) -> Option<Method> {
    sqlx::query_as::<_, (Option<String>,)>(sql("SELECT two_factor_method FROM users WHERE id = $1"))
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .and_then(|(method,)| method)
        .and_then(|method| Method::parse(&method))
}

async fn accepts_authenticator_code(state: &AppState, user_id: &str, code: &str) -> bool {
    let row = sqlx::query_as::<_, (Option<String>, i64)>(sql("SELECT totp_secret, totp_last_step FROM users WHERE id = $1"))
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();
    let Some((Some(sealed), last_step)) = row else {
        return false;
    };
    let Some(secret) = open_secret(state, &sealed) else {
        return false;
    };
    let Some(step) = totp::matching_step(&secret, code, last_step) else {
        return false;
    };
    sqlx::query(sql("UPDATE users SET totp_last_step = $1 WHERE id = $2 AND totp_last_step < $1"))
        .bind(step)
        .bind(user_id)
        .execute(&state.db)
        .await
        .is_ok_and(|result| result.rows_affected() == 1)
}

pub async fn accepts_code(state: &AppState, user_id: &str, code: &str) -> bool {
    let accepted = match method_of(state, user_id).await {
        Some(Method::Totp) => accepts_authenticator_code(state, user_id, code).await,
        Some(Method::Email) => email_tokens::consume_code(state, user_id, code, TokenPurpose::TwoFactorCode).await.is_some(),
        None => return false,
    };
    accepted || backup_codes::consume(state, user_id, code).await
}

pub async fn send_email_code(state: &AppState, user_id: &str, username: &str, email: &str) -> Result<(), ApiError> {
    if email_tokens::recently_issued(state, user_id, TokenPurpose::TwoFactorCode).await {
        return Ok(());
    }
    let code = email_tokens::issue_code(state, user_id, email, TokenPurpose::TwoFactorCode)
        .await
        .map_err(crate::server_errors::hidden)?;
    let brand = crate::email::brand::load(state).await;
    let message = crate::email::messages::sign_in_code(&brand, username, &code, SIGN_IN_CODE_MINUTES);
    state.mailer.send(email, message).await.map_err(|error| {
        tracing::warn!("Two-factor code email to {email} failed: {error}");
        (StatusCode::BAD_GATEWAY, "The code could not be emailed. Try again later.".to_string())
    })
}

pub async fn send_email_code_to_user(state: &AppState, user_id: &str) -> Result<(), ApiError> {
    let user = crate::accounts::lookup::user_by_id(state, user_id)
        .await?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;
    send_email_code(state, &user.id, &user.username, &user.email).await
}

pub async fn turn_off(state: &AppState, user_id: &str) -> Result<(), ApiError> {
    sqlx::query(sql(
        "UPDATE users SET two_factor_method = NULL, totp_secret = NULL, totp_setup_secret = NULL, totp_last_step = 0 WHERE id = $1",
    ))
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(crate::server_errors::hidden)?;
    backup_codes::remove_all(state, user_id).await
}

pub async fn reset_from_environment(state: &AppState) {
    let Ok(email) = std::env::var("TWO_FACTOR_RESET_EMAIL") else {
        return;
    };
    let email = email.trim();
    if email.is_empty() {
        return;
    }
    match crate::accounts::lookup::user_by_email(state, email).await {
        Ok(Some(user)) => match turn_off(state, &user.id).await {
            Ok(()) => tracing::warn!("Two-factor authentication was turned off for {email} because TWO_FACTOR_RESET_EMAIL is set. Remove it now."),
            Err((_, message)) => tracing::error!("Could not turn off two-factor authentication for {email}: {message}"),
        },
        _ => tracing::warn!("TWO_FACTOR_RESET_EMAIL is set, but no account uses {email}"),
    }
}

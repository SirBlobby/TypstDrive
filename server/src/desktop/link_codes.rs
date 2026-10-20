use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::session::{ensure_supported_client, issue_device_token, DeviceLoginResponse};
use super::hash_token;
use crate::accounts::client_info::ClientInfo;
use crate::db::sql;
use crate::models::User;
use crate::AppState;

const CODE_ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const CODE_LENGTH: usize = 8;
const CODE_LIFETIME_MINUTES: i64 = 10;
const LINK_CODE_SCOPE: &str = "desktop-link-code";

type ApiError = (StatusCode, String);

#[derive(Serialize)]
pub struct LinkCodeResponse {
    pub code: String,
    pub expires_at: String,
    pub expires_in_seconds: i64,
}

#[derive(Deserialize)]
pub struct ExchangeRequest {
    pub code: String,
    pub device_name: Option<String>,
    pub client_version: Option<String>,
}

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn timestamp(time: chrono::DateTime<chrono::Utc>) -> String {
    time.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn generate_code() -> String {
    let random_bytes = Uuid::new_v4();
    random_bytes
        .as_bytes()
        .iter()
        .take(CODE_LENGTH)
        .map(|byte| CODE_ALPHABET[(*byte as usize) % CODE_ALPHABET.len()] as char)
        .collect()
}

fn display_code(code: &str) -> String {
    format!("{}-{}", &code[..CODE_LENGTH / 2], &code[CODE_LENGTH / 2..])
}

fn normalize_code(input: &str) -> String {
    input
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_uppercase())
        .collect()
}

pub async fn create_link_code(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<LinkCodeResponse>, ApiError> {
    let user_id = jar
        .get("session_user_id")
        .map(|cookie| cookie.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Sign in to link Typst Desktop".to_string()))?;

    let now = chrono::Utc::now();
    sqlx::query(sql("DELETE FROM desktop_link_codes WHERE user_id = $1 OR expires_at <= $2"))
        .bind(&user_id)
        .bind(timestamp(now))
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    let code = generate_code();
    let expires_at = timestamp(now + chrono::Duration::minutes(CODE_LIFETIME_MINUTES));
    sqlx::query(sql(
        "INSERT INTO desktop_link_codes (id, user_id, code_hash, created_at, expires_at) VALUES ($1, $2, $3, $4, $5)",
    ))
    .bind(Uuid::new_v4().to_string())
    .bind(&user_id)
    .bind(hash_token(&code))
    .bind(timestamp(now))
    .bind(&expires_at)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(Json(LinkCodeResponse {
        code: display_code(&code),
        expires_at,
        expires_in_seconds: CODE_LIFETIME_MINUTES * 60,
    }))
}

async fn redeem_code(state: &AppState, code: &str) -> Result<Option<String>, sqlx::Error> {
    let code_hash = hash_token(code);
    let found = sqlx::query_as::<_, (String,)>(sql(
        "SELECT user_id FROM desktop_link_codes WHERE code_hash = $1 AND expires_at > $2",
    ))
    .bind(&code_hash)
    .bind(timestamp(chrono::Utc::now()))
    .fetch_optional(&state.db)
    .await?;
    let Some((user_id,)) = found else {
        return Ok(None);
    };

    let removed = sqlx::query(sql("DELETE FROM desktop_link_codes WHERE code_hash = $1"))
        .bind(&code_hash)
        .execute(&state.db)
        .await?
        .rows_affected();
    Ok((removed > 0).then_some(user_id))
}

pub async fn exchange_link_code(
    State(state): State<AppState>,
    client: ClientInfo,
    Json(payload): Json<ExchangeRequest>,
) -> Result<Json<DeviceLoginResponse>, ApiError> {
    ensure_supported_client(payload.client_version.as_deref())?;
    state.login_guard.ensure_allowed(LINK_CODE_SCOPE, &client.ip_address, &client.ip_address)?;

    let code = normalize_code(&payload.code);
    let user_id = match redeem_code(&state, &code).await.map_err(internal_error)? {
        Some(user_id) if code.len() == CODE_LENGTH => user_id,
        _ => {
            let settings = crate::accounts::auth_settings::load(&state).await;
            state.login_guard.record_failure(LINK_CODE_SCOPE, &client.ip_address, &client.ip_address, settings.lockout_rules());
            return Err((StatusCode::UNAUTHORIZED, "This code is invalid or has expired. Generate a new one.".to_string()));
        }
    };
    state.login_guard.record_success(LINK_CODE_SCOPE, &client.ip_address);

    let user = sqlx::query_as::<_, User>(sql(
        "SELECT id, username, email, password_hash, is_admin, email_verified FROM users WHERE id = $1",
    ))
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    let token = issue_device_token(&state, &user.id, payload.device_name).await?;
    Ok(Json(DeviceLoginResponse {
        token,
        user_id: user.id,
        username: user.username,
        email: user.email,
    }))
}

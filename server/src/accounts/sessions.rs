use axum::extract::{FromRequestParts, Path, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{DateTime, TimeDelta, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::accounts::client_info::ClientInfo;
use crate::accounts::cookies;
use crate::db::sql;
use crate::AppState;

pub const SESSION_COOKIE: &str = "typstdrive_session";
const LAST_SEEN_INTERVAL_SECONDS: i64 = 60;

type ApiError = (StatusCode, String);

#[derive(Clone)]
pub struct ActiveSession {
    pub session_id: String,
    pub user_id: String,
}

pub struct CurrentSession(pub ActiveSession);

impl<S: Send + Sync> FromRequestParts<S> for CurrentSession {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<ActiveSession>()
            .cloned()
            .map(CurrentSession)
            .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))
    }
}

#[derive(Serialize, FromRow)]
pub struct SessionView {
    pub id: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: String,
    pub last_seen_at: String,
    pub expires_at: String,
    #[sqlx(default)]
    pub current: bool,
}

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

pub fn format_time(time: DateTime<Utc>) -> String {
    time.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn generate_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

fn hash_token(token: &str) -> String {
    crate::hashing::sha256_hex(token.as_bytes())
}

fn session_cookie(token: String, session_days: i64) -> Cookie<'static> {
    let mut cookie = Cookie::new(SESSION_COOKIE, token);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    cookie.set_secure(crate::site_url::uses_https());
    cookie.set_max_age(cookie::time::Duration::days(session_days));
    cookie
}

pub fn cleared_cookie() -> Cookie<'static> {
    Cookie::build(SESSION_COOKIE).path("/").build()
}

pub async fn start_session(state: &AppState, user_id: &str, client: &ClientInfo) -> Result<Cookie<'static>, ApiError> {
    let session_days = crate::accounts::auth_settings::load(state).await.session_days;
    let token = generate_token();
    let now = Utc::now();
    let expires_at = now + TimeDelta::days(session_days);

    let _ = sqlx::query(sql("DELETE FROM user_sessions WHERE expires_at < $1"))
        .bind(format_time(now))
        .execute(&state.db)
        .await;

    sqlx::query(sql(
        "INSERT INTO user_sessions (id, user_id, token_hash, user_agent, ip_address, created_at, last_seen_at, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    ))
    .bind(Uuid::new_v4().to_string())
    .bind(user_id)
    .bind(hash_token(&token))
    .bind(&client.user_agent)
    .bind(&client.ip_address)
    .bind(format_time(now))
    .bind(format_time(now))
    .bind(format_time(expires_at))
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(session_cookie(token, session_days))
}

pub async fn end_session(state: &AppState, headers: &HeaderMap) {
    let Some(token) = cookies::read(headers, SESSION_COOKIE) else {
        return;
    };
    let _ = sqlx::query(sql("DELETE FROM user_sessions WHERE token_hash = $1"))
        .bind(hash_token(&token))
        .execute(&state.db)
        .await;
}

pub async fn end_all_sessions(state: &AppState, user_id: &str, keep_session_id: Option<&str>) -> Result<u64, ApiError> {
    let result = sqlx::query(sql("DELETE FROM user_sessions WHERE user_id = $1 AND id <> $2"))
        .bind(user_id)
        .bind(keep_session_id.unwrap_or(""))
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(result.rows_affected())
}

pub async fn resolve(state: &AppState, headers: &HeaderMap) -> Option<ActiveSession> {
    let token = cookies::read(headers, SESSION_COOKIE)?;
    let now = Utc::now();

    let (session_id, user_id, last_seen_at) = sqlx::query_as::<_, (String, String, String)>(sql(
        "SELECT id, user_id, last_seen_at FROM user_sessions WHERE token_hash = $1 AND expires_at > $2",
    ))
    .bind(hash_token(&token))
    .bind(format_time(now))
    .fetch_optional(&state.db)
    .await
    .ok()??;

    let refresh_before = format_time(now - TimeDelta::seconds(LAST_SEEN_INTERVAL_SECONDS));
    if last_seen_at < refresh_before {
        let _ = sqlx::query(sql("UPDATE user_sessions SET last_seen_at = $1 WHERE id = $2"))
            .bind(format_time(now))
            .bind(&session_id)
            .execute(&state.db)
            .await;
    }

    Some(ActiveSession { session_id, user_id })
}

pub async fn list_sessions(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
) -> Result<Json<Vec<SessionView>>, ApiError> {
    let mut sessions = sqlx::query_as::<_, SessionView>(sql(
        "SELECT id, user_agent, ip_address, created_at, last_seen_at, expires_at FROM user_sessions \
         WHERE user_id = $1 AND expires_at > $2 ORDER BY last_seen_at DESC",
    ))
    .bind(&session.user_id)
    .bind(format_time(Utc::now()))
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    for view in &mut sessions {
        view.current = view.id == session.session_id;
    }
    Ok(Json(sessions))
}

pub async fn revoke_session(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    Path(session_id): Path<String>,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), ApiError> {
    let result = sqlx::query(sql("DELETE FROM user_sessions WHERE id = $1 AND user_id = $2"))
        .bind(&session_id)
        .bind(&session.user_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Session not found".to_string()));
    }

    let jar = if session_id == session.session_id { jar.remove(cleared_cookie()) } else { jar };
    Ok((jar, StatusCode::NO_CONTENT))
}

pub async fn revoke_other_sessions(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
) -> Result<Json<serde_json::Value>, ApiError> {
    let revoked = end_all_sessions(&state, &session.user_id, Some(&session.session_id)).await?;
    Ok(Json(serde_json::json!({ "revoked": revoked })))
}

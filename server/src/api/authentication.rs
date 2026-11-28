use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::api::keys::hash_key;
use crate::db::sql;
use crate::AppState;

const RATE_WINDOW_SECONDS: u64 = 60;
const WORKSPACE_HEADER: &str = "x-workspace";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ApiScope {
    Render,
    Read,
}

impl ApiScope {
    pub const ALL: [ApiScope; 2] = [ApiScope::Render, ApiScope::Read];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Render => "render",
            Self::Read => "read",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|scope| scope.as_str() == value.trim())
    }
}

pub fn scopes_from_text(text: &str) -> Vec<ApiScope> {
    text.split(',').filter_map(ApiScope::parse).collect()
}

pub struct ApiCaller {
    pub user_id: String,
    pub account_id: String,
}

fn reject(status: StatusCode, message: &str) -> Response {
    (status, message.to_string()).into_response()
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
}

pub fn effective_rate_limit(key_rate_limit: i64, account_rate_limit: i64) -> i64 {
    if key_rate_limit > 0 { key_rate_limit.min(account_rate_limit) } else { account_rate_limit }
}

async fn within_rate_limit(state: &AppState, key_id: &str, rate_limit: i64) -> bool {
    let mut limiter = state.rate_limiter.lock().await;
    let now = std::time::Instant::now();
    let window = std::time::Duration::from_secs(RATE_WINDOW_SECONDS);
    let entry = limiter.entry(key_id.to_string()).or_insert((0u32, now));
    if now.duration_since(entry.1) > window {
        *entry = (1, now);
        true
    } else if (entry.0 as i64) < rate_limit {
        entry.0 += 1;
        true
    } else {
        false
    }
}

fn requested_workspace(headers: &HeaderMap) -> Option<String> {
    headers
        .get(WORKSPACE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && value != "personal")
}

async fn acting_account(state: &AppState, headers: &HeaderMap, user_id: &str) -> Result<String, Response> {
    let Some(team_id) = requested_workspace(headers) else {
        return Ok(user_id.to_string());
    };
    if !crate::teams::policy::load_policy(state).await.enabled {
        return Err(reject(StatusCode::FORBIDDEN, "Team spaces are turned off on this server"));
    }
    if crate::teams::membership(state, &team_id, user_id).await.is_none() {
        return Err(reject(StatusCode::FORBIDDEN, "You are not a member of this team"));
    }
    Ok(team_id)
}

pub async fn authenticate(state: &AppState, headers: &HeaderMap, scope: ApiScope) -> Result<ApiCaller, Response> {
    let token = bearer_token(headers).ok_or_else(|| {
        reject(StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header. Use: Authorization: Bearer <api-key>")
    })?;

    let row = sqlx::query_as::<_, (String, String, String, Option<String>, i64)>(sql(
        "SELECT id, user_id, scopes, expires_at, rate_limit FROM api_keys WHERE key_hash = $1",
    ))
    .bind(hash_key(&token))
    .fetch_optional(&state.db)
    .await
    .map_err(|error| {
        tracing::error!("API key lookup failed: {error}");
        reject(StatusCode::INTERNAL_SERVER_ERROR, crate::server_errors::GENERIC_MESSAGE)
    })?;

    let Some((key_id, user_id, scopes, expires_at, key_rate_limit)) = row else {
        return Err(reject(StatusCode::UNAUTHORIZED, "Invalid API key"));
    };

    if expires_at.is_some_and(|expiry| expiry <= crate::db::timestamp_now()) {
        return Err(reject(StatusCode::UNAUTHORIZED, "This API key has expired"));
    }
    if !scopes_from_text(&scopes).contains(&scope) {
        return Err(reject(
            StatusCode::FORBIDDEN,
            &format!("This API key does not have the '{}' scope", scope.as_str()),
        ));
    }

    let limits = crate::api::limits::effective_limits(state, &user_id).await;
    if !limits.api_enabled {
        return Err(reject(StatusCode::FORBIDDEN, "API access is disabled for this account"));
    }
    if limits.daily_quota > 0 && crate::api::limits::requests_today(state, &user_id).await + crate::api::usage_buffer::pending_today(&user_id) >= limits.daily_quota {
        return Err(reject(StatusCode::TOO_MANY_REQUESTS, "Daily request quota reached for this account."));
    }

    let key_limited = key_rate_limit > 0 && !within_rate_limit(state, &format!("key:{key_id}"), key_rate_limit).await;
    if key_limited || !within_rate_limit(state, &format!("account:{user_id}"), limits.rate_limit).await {
        return Err(reject(StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded. Max requests per minute reached."));
    }

    let account_id = acting_account(state, headers, &user_id).await?;
    crate::api::usage_buffer::note(&key_id, &user_id);
    Ok(ApiCaller { user_id, account_id })
}

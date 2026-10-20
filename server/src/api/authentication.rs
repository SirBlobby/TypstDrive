use std::sync::atomic::{AtomicI64, Ordering};

use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::api::keys::hash_key;
use crate::db::sql;
use crate::AppState;

const RATE_WINDOW_SECONDS: u64 = 60;
const USAGE_PRUNE_INTERVAL_SECONDS: i64 = 60;
const WORKSPACE_HEADER: &str = "x-workspace";

static LAST_USAGE_PRUNE: AtomicI64 = AtomicI64::new(0);

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

async fn record_usage(state: &AppState, key_id: &str) {
    let now = chrono::Utc::now();

    let _ = sqlx::query(sql("UPDATE api_keys SET last_used_at = $1 WHERE id = $2"))
        .bind(now.to_rfc3339())
        .bind(key_id)
        .execute(&state.db)
        .await;

    let _ = sqlx::query(sql(
        "INSERT INTO api_key_usage (key_id, date, count) VALUES ($1, $2, 1) \
         ON CONFLICT (key_id, date) DO UPDATE SET count = api_key_usage.count + 1",
    ))
    .bind(key_id)
    .bind(now.format("%Y-%m-%d").to_string())
    .execute(&state.db)
    .await;

    let _ = sqlx::query(sql(
        "INSERT INTO api_key_usage_detail (key_id, minute, count) VALUES ($1, $2, 1) \
         ON CONFLICT (key_id, minute) DO UPDATE SET count = api_key_usage_detail.count + 1",
    ))
    .bind(key_id)
    .bind(now.format("%Y-%m-%d %H:%M").to_string())
    .execute(&state.db)
    .await;

    let last_prune = LAST_USAGE_PRUNE.load(Ordering::Relaxed);
    let now_seconds = now.timestamp();
    if now_seconds - last_prune < USAGE_PRUNE_INTERVAL_SECONDS
        || LAST_USAGE_PRUNE.compare_exchange(last_prune, now_seconds, Ordering::Relaxed, Ordering::Relaxed).is_err()
    {
        return;
    }
    let cutoff_minute = (now - chrono::TimeDelta::hours(25)).format("%Y-%m-%d %H:%M").to_string();
    let _ = sqlx::query(sql("DELETE FROM api_key_usage_detail WHERE minute < $1"))
        .bind(cutoff_minute)
        .execute(&state.db)
        .await;
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
    .map_err(|error| reject(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {error}")))?;

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
    if limits.daily_quota > 0 && crate::api::limits::requests_today(state, &user_id).await >= limits.daily_quota {
        return Err(reject(StatusCode::TOO_MANY_REQUESTS, "Daily request quota reached for this account."));
    }

    let rate_limit = effective_rate_limit(key_rate_limit, limits.rate_limit);
    if !within_rate_limit(state, &key_id, rate_limit).await {
        return Err(reject(StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded. Max requests per minute reached."));
    }

    let account_id = acting_account(state, headers, &user_id).await?;
    record_usage(state, &key_id).await;
    Ok(ApiCaller { user_id, account_id })
}

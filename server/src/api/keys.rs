use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::collections::HashMap;
use axum_extra::extract::cookie::SignedCookieJar;
use uuid::Uuid;

use crate::{
    models::{ApiKeyView, CreateApiKeyRequest, UsagePoint},
    AppState,
};
use crate::api::authentication::{effective_rate_limit, ApiScope};
use crate::db::sql;

fn get_user_id(jar: &SignedCookieJar) -> Option<String> {
    jar.get("session_user_id").map(|c| c.value().to_string())
}

pub fn hash_key(key: &str) -> String {
    crate::hashing::sha256_hex(key.as_bytes())
}

const MAX_KEY_LIFETIME_DAYS: i64 = 3650;

fn requested_scopes(requested: &Option<Vec<String>>) -> Result<String, (StatusCode, String)> {
    let Some(requested) = requested else {
        return Ok(ApiScope::Render.as_str().to_string());
    };
    let mut scopes = Vec::new();
    for name in requested {
        let scope = ApiScope::parse(name).ok_or((StatusCode::BAD_REQUEST, format!("Unknown scope '{name}'")))?;
        if !scopes.contains(&scope) {
            scopes.push(scope);
        }
    }
    if scopes.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Choose at least one scope".to_string()));
    }
    Ok(scopes.iter().map(|scope| scope.as_str()).collect::<Vec<_>>().join(","))
}

fn expiry_from_days(days: Option<i64>) -> Result<Option<String>, (StatusCode, String)> {
    match days {
        None | Some(0) => Ok(None),
        Some(days) if !(1..=MAX_KEY_LIFETIME_DAYS).contains(&days) => {
            Err((StatusCode::BAD_REQUEST, format!("Expiry must be between 1 and {MAX_KEY_LIFETIME_DAYS} days")))
        }
        Some(days) => Ok(Some(crate::accounts::sessions::format_time(chrono::Utc::now() + chrono::TimeDelta::days(days)))),
    }
}

fn generate_api_key() -> String {
    format!(
        "td_{}{}",
        Uuid::new_v4().to_string().replace("-", ""),
        Uuid::new_v4().to_string().replace("-", "")
    )
}

pub async fn list_keys(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<ApiKeyView>>, (StatusCode, String)> {
    let user_id = get_user_id(&jar)
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;

    let account_rate_limit = crate::api::limits::effective_limits(&state, &user_id).await.rate_limit;
    let mut keys = sqlx::query_as::<_, ApiKeyView>(
        sql("SELECT id, name, key_prefix, created_at, last_used_at, rate_limit, scopes, expires_at FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC")
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for key in &mut keys {
        key.custom_rate_limit = (key.rate_limit > 0).then_some(key.rate_limit);
        key.rate_limit = effective_rate_limit(key.rate_limit, account_rate_limit);
    }

    Ok(Json(keys))
}

pub async fn create_key(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let user_id = get_user_id(&jar)
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;

    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Key name cannot be empty".to_string()));
    }

    let limits = crate::api::limits::effective_limits(&state, &user_id).await;
    if !limits.api_enabled {
        return Err((StatusCode::FORBIDDEN, "API access is disabled for this account".to_string()));
    }
    if crate::api::limits::key_count(&state, &user_id).await >= limits.max_keys {
        return Err((StatusCode::CONFLICT, format!("Maximum of {} API keys per account", limits.max_keys)));
    }

    let scopes = requested_scopes(&payload.scopes)?;
    let expires_at = expiry_from_days(payload.expires_in_days)?;
    let custom_rate_limit = match payload.rate_limit {
        Some(limit) if limit < 1 || limit > limits.rate_limit => {
            return Err((StatusCode::BAD_REQUEST, format!("Key rate limit must be between 1 and {}", limits.rate_limit)));
        }
        Some(limit) => limit,
        None => 0,
    };

    let key = generate_api_key();
    let hash = hash_key(&key);
    let prefix = key[..11].to_string();
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        sql("INSERT INTO api_keys (id, user_id, name, key_hash, key_prefix, created_at, rate_limit, scopes, expires_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
    )
    .bind(&id)
    .bind(&user_id)
    .bind(&payload.name)
    .bind(&hash)
    .bind(&prefix)
    .bind(&now)
    .bind(custom_rate_limit)
    .bind(&scopes)
    .bind(&expires_at)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "id": id,
        "name": payload.name,
        "key": key,
        "prefix": prefix,
        "created_at": now,
        "scopes": scopes,
        "expires_at": expires_at,
        "custom_rate_limit": (custom_rate_limit > 0).then_some(custom_rate_limit),
        "rate_limit": effective_rate_limit(custom_rate_limit, limits.rate_limit),
    }))))
}

pub async fn get_aggregate_usage(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<UsagePoint>>, (StatusCode, String)> {
    let user_id = get_user_id(&jar)
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;

    let period = params.get("period").map(|s| s.as_str()).unwrap_or("1week");

    let usage = match period {
        "1hr" => {
            let cutoff = (chrono::Utc::now() - chrono::TimeDelta::hours(1))
                .format("%Y-%m-%d %H:%M")
                .to_string();
            sqlx::query_as::<_, UsagePoint>(
                sql("SELECT minute as date, CAST(SUM(count) AS BIGINT) as count
                 FROM api_key_usage_detail
                 WHERE key_id IN (SELECT id FROM api_keys WHERE user_id = $1) AND minute >= $2
                 GROUP BY minute
                 ORDER BY minute ASC")
            )
            .bind(&user_id)
            .bind(&cutoff)
            .fetch_all(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        }
        "1day" => {
            let cutoff = (chrono::Utc::now() - chrono::TimeDelta::hours(24))
                .format("%Y-%m-%d %H:%M")
                .to_string();
            sqlx::query_as::<_, UsagePoint>(
                sql("SELECT SUBSTR(minute, 1, 13) as date, CAST(SUM(count) AS BIGINT) as count
                 FROM api_key_usage_detail
                 WHERE key_id IN (SELECT id FROM api_keys WHERE user_id = $1) AND minute >= $2
                 GROUP BY SUBSTR(minute, 1, 13)
                 ORDER BY date ASC")
            )
            .bind(&user_id)
            .bind(&cutoff)
            .fetch_all(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        }
        _ => {
            let cutoff = (chrono::Utc::now() - chrono::TimeDelta::days(6))
                .format("%Y-%m-%d")
                .to_string();
            sqlx::query_as::<_, UsagePoint>(
                sql("SELECT date, CAST(SUM(count) AS BIGINT) as count
                 FROM api_key_usage
                 WHERE key_id IN (SELECT id FROM api_keys WHERE user_id = $1) AND date >= $2
                 GROUP BY date
                 ORDER BY date ASC")
            )
            .bind(&user_id)
            .bind(&cutoff)
            .fetch_all(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        }
    };

    Ok(Json(usage))
}

pub async fn regenerate_key(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(key_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = get_user_id(&jar)
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;

    let row: Option<(String,)> = sqlx::query_as(
        sql("SELECT name FROM api_keys WHERE id = $1 AND user_id = $2")
    )
    .bind(&key_id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (name,) = row.ok_or((StatusCode::NOT_FOUND, "API key not found".to_string()))?;

    let new_key = generate_api_key();
    let new_hash = hash_key(&new_key);
    let new_prefix = new_key[..11].to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        sql("UPDATE api_keys SET key_hash = $1, key_prefix = $2, created_at = $3, last_used_at = NULL WHERE id = $4 AND user_id = $5")
    )
    .bind(&new_hash)
    .bind(&new_prefix)
    .bind(&now)
    .bind(&key_id)
    .bind(&user_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "id": key_id,
        "name": name,
        "key": new_key,
        "prefix": new_prefix,
        "created_at": now,
        "rate_limit": crate::api::limits::effective_limits(&state, &user_id).await.rate_limit,
    })))
}

pub async fn delete_key(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(key_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = get_user_id(&jar)
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;

    let result = sqlx::query(sql("DELETE FROM api_keys WHERE id = $1 AND user_id = $2"))
        .bind(&key_id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "API key not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

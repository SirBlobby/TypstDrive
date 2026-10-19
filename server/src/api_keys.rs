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

fn get_user_id(jar: &SignedCookieJar) -> Option<String> {
    jar.get("session_user_id").map(|c| c.value().to_string())
}

pub fn hash_key(key: &str) -> String {
    crate::hashing::sha256_hex(key.as_bytes())
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

    let rate_limit = crate::api_limits::effective_limits(&state, &user_id).await.rate_limit;
    let mut keys = sqlx::query_as::<_, ApiKeyView>(
        "SELECT id, name, key_prefix, created_at, last_used_at, rate_limit FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for key in &mut keys {
        key.rate_limit = rate_limit;
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

    let limits = crate::api_limits::effective_limits(&state, &user_id).await;
    if !limits.api_enabled {
        return Err((StatusCode::FORBIDDEN, "API access is disabled for this account".to_string()));
    }
    if crate::api_limits::key_count(&state, &user_id).await >= limits.max_keys {
        return Err((StatusCode::CONFLICT, format!("Maximum of {} API keys per account", limits.max_keys)));
    }

    let key = generate_api_key();
    let hash = hash_key(&key);
    let prefix = key[..11].to_string(); // "td_" + 8 hex chars
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO api_keys (id, user_id, name, key_hash, key_prefix, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(&id)
    .bind(&user_id)
    .bind(&payload.name)
    .bind(&hash)
    .bind(&prefix)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "id": id,
        "name": payload.name,
        "key": key,
        "prefix": prefix,
        "created_at": now,
        "rate_limit": crate::api_limits::effective_limits(&state, &user_id).await.rate_limit,
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
                "SELECT minute as date, CAST(SUM(count) AS BIGINT) as count
                 FROM api_key_usage_detail
                 WHERE key_id IN (SELECT id FROM api_keys WHERE user_id = $1) AND minute >= $2
                 GROUP BY minute
                 ORDER BY minute ASC"
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
                "SELECT SUBSTR(minute, 1, 13) as date, CAST(SUM(count) AS BIGINT) as count
                 FROM api_key_usage_detail
                 WHERE key_id IN (SELECT id FROM api_keys WHERE user_id = $1) AND minute >= $2
                 GROUP BY SUBSTR(minute, 1, 13)
                 ORDER BY date ASC"
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
                "SELECT date, CAST(SUM(count) AS BIGINT) as count
                 FROM api_key_usage
                 WHERE key_id IN (SELECT id FROM api_keys WHERE user_id = $1) AND date >= $2
                 GROUP BY date
                 ORDER BY date ASC"
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
        "SELECT name FROM api_keys WHERE id = $1 AND user_id = $2"
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
        "UPDATE api_keys SET key_hash = $1, key_prefix = $2, created_at = $3, last_used_at = NULL WHERE id = $4 AND user_id = $5"
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
        "rate_limit": crate::api_limits::effective_limits(&state, &user_id).await.rate_limit,
    })))
}

pub async fn delete_key(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(key_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = get_user_id(&jar)
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;

    let result = sqlx::query("DELETE FROM api_keys WHERE id = $1 AND user_id = $2")
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

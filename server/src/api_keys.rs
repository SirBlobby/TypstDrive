use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::collections::HashMap;
use axum_extra::extract::cookie::SignedCookieJar;
use sha2::{Sha256, Digest};
use uuid::Uuid;

use crate::{
    models::{ApiKeyView, CreateApiKeyRequest, UsagePoint},
    AppState,
};

fn get_user_id(jar: &SignedCookieJar) -> Option<String> {
    jar.get("session_user_id").map(|c| c.value().to_string())
}

pub fn hash_key(key: &str) -> String {
    format!("{:x}", Sha256::digest(key.as_bytes()))
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

    let keys = sqlx::query_as::<_, ApiKeyView>(
        "SELECT id, name, key_prefix, created_at, last_used_at, rate_limit FROM api_keys WHERE user_id = ? ORDER BY created_at DESC"
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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

    let existing: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM api_keys WHERE user_id = ?")
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some((count,)) = existing {
        if count >= 10 {
            return Err((StatusCode::CONFLICT, "Maximum of 10 API keys per account".to_string()));
        }
    }

    let key = generate_api_key();
    let hash = hash_key(&key);
    let prefix = key[..11].to_string(); // "td_" + 8 hex chars
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO api_keys (id, user_id, name, key_hash, key_prefix, created_at) VALUES (?, ?, ?, ?, ?, ?)"
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
        "rate_limit": 60,
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
                "SELECT minute as date, SUM(count) as count
                 FROM api_key_usage_detail
                 WHERE key_id IN (SELECT id FROM api_keys WHERE user_id = ?) AND minute >= ?
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
                "SELECT SUBSTR(minute, 1, 13) as date, SUM(count) as count
                 FROM api_key_usage_detail
                 WHERE key_id IN (SELECT id FROM api_keys WHERE user_id = ?) AND minute >= ?
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
                "SELECT date, SUM(count) as count
                 FROM api_key_usage
                 WHERE key_id IN (SELECT id FROM api_keys WHERE user_id = ?) AND date >= ?
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
        "SELECT name FROM api_keys WHERE id = ? AND user_id = ?"
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
        "UPDATE api_keys SET key_hash = ?, key_prefix = ?, created_at = ?, last_used_at = NULL WHERE id = ? AND user_id = ?"
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
        "rate_limit": 60,
    })))
}

pub async fn delete_key(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(key_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = get_user_id(&jar)
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;

    let result = sqlx::query("DELETE FROM api_keys WHERE id = ? AND user_id = ?")
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

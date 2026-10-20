use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::accounts::client_info::ClientInfo;
use crate::admin::users::require_admin;
use crate::db::{sql, timestamp_now};
use crate::AppState;

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 200;

#[derive(Serialize, FromRow)]
pub struct AuditEntry {
    pub id: String,
    pub actor_id: Option<String>,
    pub actor_name: String,
    pub action: String,
    pub target: Option<String>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct AuditQuery {
    pub limit: Option<i64>,
    pub before: Option<String>,
    pub action: Option<String>,
}

#[derive(Serialize)]
pub struct AuditPage {
    pub entries: Vec<AuditEntry>,
    pub next_before: Option<String>,
}

fn optional_text(value: &str) -> Option<&str> {
    Some(value.trim()).filter(|text| !text.is_empty())
}

pub async fn record(state: &AppState, actor_id: &str, client: &ClientInfo, action: &str, target: &str, details: &str) {
    let actor_name = sqlx::query_as::<_, (String,)>(sql("SELECT username FROM users WHERE id = $1"))
        .bind(actor_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .map(|(username,)| username)
        .unwrap_or_else(|| "unknown".to_string());

    let inserted = sqlx::query(sql(
        "INSERT INTO audit_log (id, actor_id, actor_name, action, target, details, ip_address, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    ))
    .bind(Uuid::new_v4().to_string())
    .bind(actor_id)
    .bind(&actor_name)
    .bind(action)
    .bind(optional_text(target))
    .bind(optional_text(details))
    .bind(&client.ip_address)
    .bind(timestamp_now())
    .execute(&state.db)
    .await;

    if let Err(error) = inserted {
        tracing::warn!("Failed to write audit log entry for {action}: {error}");
    }
}

pub async fn list_entries(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Query(query): Query<AuditQuery>,
) -> Result<Json<AuditPage>, (StatusCode, String)> {
    require_admin(&state, &jar).await?;

    let limit = query.limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
    let before = query.before.filter(|value| !value.is_empty()).unwrap_or_else(|| "9999-12-31 23:59:59".to_string());
    let action_prefix = format!("{}%", query.action.unwrap_or_default().trim());

    let entries = sqlx::query_as::<_, AuditEntry>(sql(
        "SELECT id, actor_id, actor_name, action, target, details, ip_address, created_at FROM audit_log \
         WHERE created_at < $1 AND action LIKE $2 ORDER BY created_at DESC, id DESC LIMIT $3",
    ))
    .bind(&before)
    .bind(&action_prefix)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    let next_before = if entries.len() as i64 == limit {
        entries.last().map(|entry| entry.created_at.clone())
    } else {
        None
    };

    Ok(Json(AuditPage { entries, next_before }))
}

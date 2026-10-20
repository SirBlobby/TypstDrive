use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};

use crate::admin::users::require_admin;
use crate::AppState;
use crate::db::sql;

const DEFAULT_QUOTA_KEY: &str = "storage.default_quota";
const MAX_QUOTA_BYTES: i64 = 1024 * 1024 * 1024 * 1024 * 1024;

type ApiError = (StatusCode, String);

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct StorageUsage {
    pub documents_size_bytes: i64,
    pub files_size_bytes: i64,
    pub projects_size_bytes: i64,
    pub total_size_bytes: i64,
}

impl StorageUsage {
    fn new(documents_size_bytes: i64, files_size_bytes: i64, projects_size_bytes: i64) -> Self {
        Self {
            documents_size_bytes,
            files_size_bytes,
            projects_size_bytes,
            total_size_bytes: documents_size_bytes + files_size_bytes + projects_size_bytes,
        }
    }
}

#[derive(Serialize)]
pub struct OwnStorage {
    #[serde(flatten)]
    pub usage: StorageUsage,
    pub quota_bytes: i64,
}

#[derive(Serialize)]
pub struct UserStorage {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub usage: StorageUsage,
    pub quota_override: Option<i64>,
    pub quota_bytes: i64,
}

#[derive(Serialize)]
pub struct StorageOverview {
    pub default_quota_bytes: i64,
    pub total_size_bytes: i64,
    pub users: Vec<UserStorage>,
}

#[derive(Deserialize)]
pub struct DefaultQuotaRequest {
    pub default_quota_bytes: i64,
}

#[derive(Deserialize)]
pub struct UserQuotaRequest {
    pub quota_bytes: Option<i64>,
}

#[derive(Serialize)]
pub struct UserQuotaResponse {
    pub quota_override: Option<i64>,
    pub quota_bytes: i64,
}

const DOCUMENTS_SIZE_SQL: &str =
    "SELECT CAST(COALESCE(SUM(LENGTH(content)), 0) AS BIGINT) FROM documents WHERE owner_id = $1";
const FILES_SIZE_SQL: &str =
    "SELECT CAST(COALESCE(SUM(size_bytes), 0) AS BIGINT) FROM files WHERE owner_id = $1";
const PROJECTS_SIZE_SQL: &str =
    "SELECT CAST(COALESCE(SUM(LENGTH(pf.content)), 0) AS BIGINT) FROM project_files pf \
     INNER JOIN projects p ON p.id = pf.project_id WHERE p.owner_id = $1";

async fn sum_for_user(state: &AppState, statement: &'static str, user_id: &str) -> i64 {
    sqlx::query_as::<_, (i64,)>(sql(statement))
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map(|(size,)| size)
        .unwrap_or(0)
}

pub async fn usage_for(state: &AppState, user_id: &str) -> StorageUsage {
    StorageUsage::new(
        sum_for_user(state, DOCUMENTS_SIZE_SQL, user_id).await,
        sum_for_user(state, FILES_SIZE_SQL, user_id).await,
        sum_for_user(state, PROJECTS_SIZE_SQL, user_id).await,
    )
}

pub async fn project_file_size(state: &AppState, project_id: &str, path: &str) -> i64 {
    sqlx::query_as::<_, (i64,)>(
        sql("SELECT CAST(COALESCE(LENGTH(content), 0) AS BIGINT) FROM project_files WHERE project_id = $1 AND path = $2")
    )
    .bind(project_id)
    .bind(path)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None)
    .map(|(size,)| size)
    .unwrap_or(0)
}

pub async fn default_quota(state: &AppState) -> i64 {
    crate::admin::site_settings::load_settings(state, "storage.")
        .await
        .get(DEFAULT_QUOTA_KEY)
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

async fn quota_override(state: &AppState, user_id: &str) -> Option<i64> {
    sqlx::query_as::<_, (i64,)>(sql("SELECT quota_bytes FROM user_storage_quotas WHERE user_id = $1"))
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None)
        .map(|(quota,)| quota)
}

pub async fn quota_for(state: &AppState, user_id: &str) -> i64 {
    match quota_override(state, user_id).await {
        Some(quota) => quota,
        None => default_quota(state).await,
    }
}

fn format_bytes(bytes: i64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit_index = 0;
    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }
    if unit_index == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit_index])
    }
}

pub async fn ensure_room(state: &AppState, owner_id: &str, additional_bytes: i64) -> Result<(), ApiError> {
    if additional_bytes <= 0 {
        return Ok(());
    }

    let quota = quota_for(state, owner_id).await;
    if quota <= 0 {
        return Ok(());
    }

    let used = usage_for(state, owner_id).await.total_size_bytes;
    if used + additional_bytes > quota {
        return Err((
            StatusCode::INSUFFICIENT_STORAGE,
            format!(
                "Storage quota exceeded: {} of {} used, this upload needs {} more.",
                format_bytes(used),
                format_bytes(quota),
                format_bytes(additional_bytes)
            ),
        ));
    }
    Ok(())
}

fn validate_quota(quota: i64) -> Result<(), ApiError> {
    if quota < 0 || quota > MAX_QUOTA_BYTES {
        return Err((StatusCode::BAD_REQUEST, "Quota must be between 0 and 1 PB".to_string()));
    }
    Ok(())
}

pub async fn own_storage(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<OwnStorage>, ApiError> {
    let user_id = jar
        .get("session_user_id")
        .map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    Ok(Json(OwnStorage {
        usage: usage_for(&state, &user_id).await,
        quota_bytes: quota_for(&state, &user_id).await,
    }))
}

pub async fn overview(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<StorageOverview>, ApiError> {
    require_admin(&state, &jar).await?;
    let default_quota_bytes = default_quota(&state).await;

    type OverviewRow = (String, String, String, Option<i64>, i64, i64, i64);
    let rows: Vec<OverviewRow> = sqlx::query_as(
        sql("SELECT u.id, u.username, COALESCE(u.email, ''), q.quota_bytes, \
                CAST((SELECT COALESCE(SUM(LENGTH(d.content)), 0) FROM documents d WHERE d.owner_id = u.id) AS BIGINT), \
                CAST((SELECT COALESCE(SUM(f.size_bytes), 0) FROM files f WHERE f.owner_id = u.id) AS BIGINT), \
                CAST((SELECT COALESCE(SUM(LENGTH(pf.content)), 0) FROM project_files pf \
                      INNER JOIN projects p ON p.id = pf.project_id WHERE p.owner_id = u.id) AS BIGINT) \
         FROM users u LEFT JOIN user_storage_quotas q ON q.user_id = u.id \
         ORDER BY u.username ASC")
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let users: Vec<UserStorage> = rows
        .into_iter()
        .map(|(user_id, username, email, quota_override, documents, files, projects)| UserStorage {
            user_id,
            username,
            email,
            usage: StorageUsage::new(documents, files, projects),
            quota_override,
            quota_bytes: quota_override.unwrap_or(default_quota_bytes),
        })
        .collect();

    let total_size_bytes = users.iter().map(|user| user.usage.total_size_bytes).sum();

    Ok(Json(StorageOverview { default_quota_bytes, total_size_bytes, users }))
}

pub async fn update_default_quota(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<DefaultQuotaRequest>,
) -> Result<StatusCode, ApiError> {
    require_admin(&state, &jar).await?;
    validate_quota(payload.default_quota_bytes)?;

    crate::admin::site_settings::save_settings(&state, &[(DEFAULT_QUOTA_KEY, payload.default_quota_bytes.to_string())])
        .await
        .map_err(internal_error)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_user_quota(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(user_id): Path<String>,
    Json(payload): Json<UserQuotaRequest>,
) -> Result<Json<UserQuotaResponse>, ApiError> {
    require_admin(&state, &jar).await?;

    let user_exists = sqlx::query_as::<_, (String,)>(sql("SELECT id FROM users WHERE id = $1"))
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;
    if user_exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
    }

    match payload.quota_bytes {
        Some(quota) => {
            validate_quota(quota)?;
            sqlx::query(
                sql("INSERT INTO user_storage_quotas (user_id, quota_bytes, updated_at) VALUES ($1, $2, $3) \
                 ON CONFLICT (user_id) DO UPDATE SET quota_bytes = excluded.quota_bytes, updated_at = excluded.updated_at")
            )
            .bind(&user_id)
            .bind(quota)
            .bind(crate::db::timestamp_now())
            .execute(&state.db)
            .await
            .map_err(internal_error)?;
        }
        None => {
            sqlx::query(sql("DELETE FROM user_storage_quotas WHERE user_id = $1"))
                .bind(&user_id)
                .execute(&state.db)
                .await
                .map_err(internal_error)?;
        }
    }

    Ok(Json(UserQuotaResponse {
        quota_override: payload.quota_bytes,
        quota_bytes: quota_for(&state, &user_id).await,
    }))
}

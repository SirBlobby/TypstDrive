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
const DEFAULT_TEAM_QUOTA_KEY: &str = "storage.default_team_quota";
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
    pub is_team: bool,
    pub usage: StorageUsage,
    pub quota_override: Option<i64>,
    pub quota_bytes: i64,
}

#[derive(Serialize)]
pub struct StorageOverview {
    pub default_quota_bytes: i64,
    pub default_team_quota_bytes: i64,
    pub total_size_bytes: i64,
    pub users: Vec<UserStorage>,
}

#[derive(Deserialize)]
pub struct DefaultQuotaRequest {
    pub default_quota_bytes: Option<i64>,
    pub default_team_quota_bytes: Option<i64>,
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

const USAGE_SQL: &str = "SELECT \
     CAST(COALESCE((SELECT SUM(LENGTH(content)) FROM documents WHERE owner_id = $1), 0) AS BIGINT), \
     CAST(COALESCE((SELECT SUM(size_bytes) FROM files WHERE owner_id = $2), 0) AS BIGINT), \
     CAST(COALESCE((SELECT SUM(LENGTH(pf.content)) FROM project_files pf \
         INNER JOIN projects p ON p.id = pf.project_id WHERE p.owner_id = $3), 0) AS BIGINT)";

pub async fn usage_for(state: &AppState, user_id: &str) -> StorageUsage {
    let (documents, files, projects) = sqlx::query_as::<_, (i64, i64, i64)>(sql(USAGE_SQL))
        .bind(user_id)
        .bind(user_id)
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or((0, 0, 0));
    StorageUsage::new(documents, files, projects)
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

pub struct DefaultQuotas {
    pub users: i64,
    pub teams: i64,
}

impl DefaultQuotas {
    fn for_account(&self, is_team: bool) -> i64 {
        if is_team { self.teams } else { self.users }
    }
}

pub async fn default_quotas(state: &AppState) -> DefaultQuotas {
    let settings = crate::admin::site_settings::load_settings(state, "storage.").await;
    let read = |key: &str| settings.get(key).and_then(|value| value.parse().ok()).unwrap_or(0);
    DefaultQuotas { users: read(DEFAULT_QUOTA_KEY), teams: read(DEFAULT_TEAM_QUOTA_KEY) }
}

async fn is_team_account(state: &AppState, account_id: &str) -> bool {
    sqlx::query_as::<_, (String,)>(sql("SELECT account_kind FROM users WHERE id = $1"))
        .bind(account_id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None)
        .is_some_and(|(account_kind,)| account_kind == "team")
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
        None => default_quotas(state).await.for_account(is_team_account(state, user_id).await),
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
    let defaults = default_quotas(&state).await;

    type OverviewRow = (String, String, String, String, Option<i64>, i64, i64, i64);
    let rows: Vec<OverviewRow> = sqlx::query_as(
        sql("SELECT u.id, u.username, COALESCE(u.email, 'Team space'), u.account_kind, q.quota_bytes, \
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
        .map(|(user_id, username, email, account_kind, quota_override, documents, files, projects)| {
            let is_team = account_kind == "team";
            UserStorage {
                user_id,
                username,
                email,
                is_team,
                usage: StorageUsage::new(documents, files, projects),
                quota_override,
                quota_bytes: quota_override.unwrap_or(defaults.for_account(is_team)),
            }
        })
        .collect();

    let total_size_bytes = users.iter().map(|user| user.usage.total_size_bytes).sum();

    Ok(Json(StorageOverview {
        default_quota_bytes: defaults.users,
        default_team_quota_bytes: defaults.teams,
        total_size_bytes,
        users,
    }))
}

pub async fn update_default_quota(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: crate::accounts::client_info::ClientInfo,
    Json(payload): Json<DefaultQuotaRequest>,
) -> Result<StatusCode, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    let mut changes = Vec::new();
    if let Some(quota) = payload.default_quota_bytes {
        validate_quota(quota)?;
        changes.push((DEFAULT_QUOTA_KEY, quota.to_string()));
    }
    if let Some(quota) = payload.default_team_quota_bytes {
        validate_quota(quota)?;
        changes.push((DEFAULT_TEAM_QUOTA_KEY, quota.to_string()));
    }

    crate::admin::site_settings::save_settings(&state, &changes)
        .await
        .map_err(internal_error)?;

    for (key, value) in &changes {
        let action = if *key == DEFAULT_TEAM_QUOTA_KEY { "settings.storage_default_team_quota" } else { "settings.storage_default_quota" };
        crate::admin::audit::record(&state, &admin_id, &client, action, "", value).await;
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_user_quota(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: crate::accounts::client_info::ClientInfo,
    Path(user_id): Path<String>,
    Json(payload): Json<UserQuotaRequest>,
) -> Result<Json<UserQuotaResponse>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;

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

    crate::admin::audit::record(&state, &admin_id, &client, "storage.user_quota", &user_id, &format!("{:?}", payload.quota_bytes)).await;

    Ok(Json(UserQuotaResponse {
        quota_override: payload.quota_bytes,
        quota_bytes: quota_for(&state, &user_id).await,
    }))
}

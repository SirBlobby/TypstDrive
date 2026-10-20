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

const ENABLED_KEY: &str = "api.enabled";
const RATE_LIMIT_KEY: &str = "api.rate_limit";
const MAX_KEYS_KEY: &str = "api.max_keys";
const DAILY_QUOTA_KEY: &str = "api.daily_quota";

const MAX_RATE_LIMIT: i64 = 100_000;
const MAX_KEYS_LIMIT: i64 = 1_000;
const MAX_DAILY_QUOTA: i64 = 100_000_000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ApiLimits {
    pub api_enabled: bool,
    pub rate_limit: i64,
    pub max_keys: i64,
    pub daily_quota: i64,
}

impl Default for ApiLimits {
    fn default() -> Self {
        Self { api_enabled: true, rate_limit: 60, max_keys: 10, daily_quota: 0 }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ApiLimitOverrides {
    pub api_enabled: Option<bool>,
    pub rate_limit: Option<i64>,
    pub max_keys: Option<i64>,
    pub daily_quota: Option<i64>,
}

impl ApiLimitOverrides {
    fn is_empty(&self) -> bool {
        self.api_enabled.is_none() && self.rate_limit.is_none() && self.max_keys.is_none() && self.daily_quota.is_none()
    }

    fn apply_to(&self, defaults: ApiLimits) -> ApiLimits {
        ApiLimits {
            api_enabled: self.api_enabled.unwrap_or(defaults.api_enabled),
            rate_limit: self.rate_limit.unwrap_or(defaults.rate_limit),
            max_keys: self.max_keys.unwrap_or(defaults.max_keys),
            daily_quota: self.daily_quota.unwrap_or(defaults.daily_quota),
        }
    }
}

#[derive(Serialize)]
pub struct UserApiAccess {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    pub overrides: ApiLimitOverrides,
    pub effective: ApiLimits,
    pub key_count: i64,
    pub requests_today: i64,
}

#[derive(Serialize)]
pub struct ApiAccessOverview {
    pub defaults: ApiLimits,
    pub users: Vec<UserApiAccess>,
}

#[derive(Serialize)]
pub struct OwnApiLimits {
    #[serde(flatten)]
    pub limits: ApiLimits,
    pub key_count: i64,
    pub requests_today: i64,
}

type ApiError = (StatusCode, String);

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

pub fn today() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

fn validate(name: &str, value: Option<i64>, minimum: i64, maximum: i64) -> Result<(), ApiError> {
    match value {
        Some(number) if number < minimum || number > maximum => Err((
            StatusCode::BAD_REQUEST,
            format!("{name} must be between {minimum} and {maximum}"),
        )),
        _ => Ok(()),
    }
}

fn validate_limits(rate_limit: Option<i64>, max_keys: Option<i64>, daily_quota: Option<i64>) -> Result<(), ApiError> {
    validate("Rate limit", rate_limit, 1, MAX_RATE_LIMIT)?;
    validate("Maximum keys", max_keys, 0, MAX_KEYS_LIMIT)?;
    validate("Daily quota", daily_quota, 0, MAX_DAILY_QUOTA)
}

pub async fn load_defaults(state: &AppState) -> ApiLimits {
    let rows = crate::admin::site_settings::load_settings(state, "api.").await;

    let mut defaults = ApiLimits::default();
    for (key, value) in rows {
        match key.as_str() {
            ENABLED_KEY => defaults.api_enabled = value == "true",
            RATE_LIMIT_KEY => defaults.rate_limit = value.parse().unwrap_or(defaults.rate_limit),
            MAX_KEYS_KEY => defaults.max_keys = value.parse().unwrap_or(defaults.max_keys),
            DAILY_QUOTA_KEY => defaults.daily_quota = value.parse().unwrap_or(defaults.daily_quota),
            _ => {}
        }
    }
    defaults
}

async fn load_overrides(state: &AppState, user_id: &str) -> ApiLimitOverrides {
    let row: Option<(Option<i64>, Option<i64>, Option<i64>, Option<i64>)> = sqlx::query_as(
        sql("SELECT api_enabled, rate_limit, max_keys, daily_quota FROM user_api_limits WHERE user_id = $1")
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    match row {
        Some((api_enabled, rate_limit, max_keys, daily_quota)) => ApiLimitOverrides {
            api_enabled: api_enabled.map(|value| value != 0),
            rate_limit,
            max_keys,
            daily_quota,
        },
        None => ApiLimitOverrides::default(),
    }
}

pub async fn effective_limits(state: &AppState, user_id: &str) -> ApiLimits {
    let defaults = load_defaults(state).await;
    load_overrides(state, user_id).await.apply_to(defaults)
}

pub async fn requests_today(state: &AppState, user_id: &str) -> i64 {
    sqlx::query_as::<_, (i64,)>(
        sql("SELECT CAST(COALESCE(SUM(daily_usage.count), 0) AS BIGINT) FROM api_key_usage daily_usage \
         INNER JOIN api_keys k ON k.id = daily_usage.key_id \
         WHERE k.user_id = $1 AND daily_usage.date = $2")
    )
    .bind(user_id)
    .bind(today())
    .fetch_one(&state.db)
    .await
    .map(|(count,)| count)
    .unwrap_or(0)
}

pub async fn key_count(state: &AppState, user_id: &str) -> i64 {
    sqlx::query_as::<_, (i64,)>(sql("SELECT COUNT(*) FROM api_keys WHERE user_id = $1"))
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map(|(count,)| count)
        .unwrap_or(0)
}

pub async fn own_limits(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<OwnApiLimits>, ApiError> {
    let user_id = jar
        .get("session_user_id")
        .map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    Ok(Json(OwnApiLimits {
        limits: effective_limits(&state, &user_id).await,
        key_count: key_count(&state, &user_id).await,
        requests_today: requests_today(&state, &user_id).await,
    }))
}

pub async fn overview(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<ApiAccessOverview>, ApiError> {
    require_admin(&state, &jar).await?;
    let defaults = load_defaults(&state).await;

    type OverviewRow = (String, String, String, i64, Option<i64>, Option<i64>, Option<i64>, Option<i64>, i64, i64);
    let rows: Vec<OverviewRow> = sqlx::query_as(
        sql("SELECT u.id, u.username, COALESCE(u.email, ''), u.is_admin, \
                l.api_enabled, l.rate_limit, l.max_keys, l.daily_quota, \
                CAST((SELECT COUNT(*) FROM api_keys k WHERE k.user_id = u.id) AS BIGINT), \
                CAST((SELECT COALESCE(SUM(daily_usage.count), 0) FROM api_key_usage daily_usage \
                      INNER JOIN api_keys k ON k.id = daily_usage.key_id \
                      WHERE k.user_id = u.id AND daily_usage.date = $1) AS BIGINT) \
         FROM users u LEFT JOIN user_api_limits l ON l.user_id = u.id \
         ORDER BY u.username ASC")
    )
    .bind(today())
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let users = rows
        .into_iter()
        .map(|(user_id, username, email, is_admin, api_enabled, rate_limit, max_keys, daily_quota, key_count, requests_today)| {
            let overrides = ApiLimitOverrides {
                api_enabled: api_enabled.map(|value| value != 0),
                rate_limit,
                max_keys,
                daily_quota,
            };
            UserApiAccess {
                user_id,
                username,
                email,
                is_admin: is_admin != 0,
                effective: overrides.apply_to(defaults),
                overrides,
                key_count,
                requests_today,
            }
        })
        .collect();

    Ok(Json(ApiAccessOverview { defaults, users }))
}

pub async fn update_defaults(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<ApiLimits>,
) -> Result<Json<ApiLimits>, ApiError> {
    require_admin(&state, &jar).await?;
    validate_limits(Some(payload.rate_limit), Some(payload.max_keys), Some(payload.daily_quota))?;

    let entries = [
        (ENABLED_KEY, payload.api_enabled.to_string()),
        (RATE_LIMIT_KEY, payload.rate_limit.to_string()),
        (MAX_KEYS_KEY, payload.max_keys.to_string()),
        (DAILY_QUOTA_KEY, payload.daily_quota.to_string()),
    ];

    crate::admin::site_settings::save_settings(&state, &entries).await.map_err(internal_error)?;

    Ok(Json(payload))
}

pub async fn update_user_limits(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(user_id): Path<String>,
    Json(payload): Json<ApiLimitOverrides>,
) -> Result<Json<ApiLimits>, ApiError> {
    require_admin(&state, &jar).await?;
    validate_limits(payload.rate_limit, payload.max_keys, payload.daily_quota)?;

    let user_exists = sqlx::query_as::<_, (String,)>(sql("SELECT id FROM users WHERE id = $1"))
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;
    if user_exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
    }

    if payload.is_empty() {
        sqlx::query(sql("DELETE FROM user_api_limits WHERE user_id = $1"))
            .bind(&user_id)
            .execute(&state.db)
            .await
            .map_err(internal_error)?;
    } else {
        sqlx::query(
            sql("INSERT INTO user_api_limits (user_id, api_enabled, rate_limit, max_keys, daily_quota, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (user_id) DO UPDATE SET api_enabled = excluded.api_enabled, rate_limit = excluded.rate_limit, \
             max_keys = excluded.max_keys, daily_quota = excluded.daily_quota, updated_at = excluded.updated_at")
        )
        .bind(&user_id)
        .bind(payload.api_enabled.map(|enabled| if enabled { 1i64 } else { 0i64 }))
        .bind(payload.rate_limit)
        .bind(payload.max_keys)
        .bind(payload.daily_quota)
        .bind(crate::db::timestamp_now())
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    }

    Ok(Json(effective_limits(&state, &user_id).await))
}

use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};

use crate::admin::users::require_admin;
use crate::admin::site_settings::{load_settings, save_settings};
use crate::AppState;
use crate::db::sql;

const PUBLISHING_ENABLED_KEY: &str = "packages.publishing_enabled";
const ADMINS_ONLY_KEY: &str = "packages.admins_only";
const MAX_SIZE_KEY: &str = "packages.max_package_size_bytes";
const MAX_VERSIONS_KEY: &str = "packages.max_versions_per_package";
const OWNER_DELETE_KEY: &str = "packages.allow_owner_delete";
const PRIVATE_PACKAGES_KEY: &str = "packages.allow_private_packages";
const DEFAULT_VISIBILITY_KEY: &str = "packages.default_visibility";

pub const PUBLIC: &str = "public";
pub const PRIVATE: &str = "private";

type ApiError = (StatusCode, String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagePolicy {
    pub publishing_enabled: bool,
    pub admins_only: bool,
    pub max_package_size_bytes: i64,
    pub max_versions_per_package: i64,
    pub allow_owner_delete: bool,
    pub allow_private_packages: bool,
    pub default_visibility: String,
}

#[derive(Serialize)]
pub struct UserPackagePolicy {
    #[serde(flatten)]
    pub policy: PackagePolicy,
    pub can_publish: bool,
    pub can_delete_own: bool,
    pub can_publish_private: bool,
}

pub fn parse_visibility(value: &str) -> Result<&'static str, ApiError> {
    match value.trim() {
        PUBLIC => Ok(PUBLIC),
        PRIVATE => Ok(PRIVATE),
        _ => Err((StatusCode::BAD_REQUEST, "Visibility must be public or private".to_string())),
    }
}

impl PackagePolicy {
    pub fn check_can_publish(&self, is_admin: bool) -> Result<(), ApiError> {
        if is_admin {
            return Ok(());
        }
        if !self.publishing_enabled {
            return Err((StatusCode::FORBIDDEN, "Package publishing is disabled on this instance".to_string()));
        }
        if self.admins_only {
            return Err((StatusCode::FORBIDDEN, "Only administrators can publish packages".to_string()));
        }
        Ok(())
    }

    pub fn check_visibility(&self, visibility: &str, is_admin: bool) -> Result<(), ApiError> {
        if visibility == PRIVATE && !is_admin && !self.allow_private_packages {
            return Err((StatusCode::FORBIDDEN, "Private packages are turned off on this instance".to_string()));
        }
        Ok(())
    }

    pub fn visibility_for_new_package(&self, requested: Option<&str>, is_admin: bool) -> Result<&'static str, ApiError> {
        let visibility = match requested {
            Some(value) => parse_visibility(value)?,
            None if self.default_visibility == PRIVATE && (is_admin || self.allow_private_packages) => PRIVATE,
            None => PUBLIC,
        };
        self.check_visibility(visibility, is_admin)?;
        Ok(visibility)
    }

    pub fn check_package_size(&self, size_bytes: i64) -> Result<(), ApiError> {
        if self.max_package_size_bytes > 0 && size_bytes > self.max_package_size_bytes {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                format!("Package is {} bytes; the limit is {} bytes", size_bytes, self.max_package_size_bytes),
            ));
        }
        Ok(())
    }

    pub fn check_version_count(&self, existing_versions: i64) -> Result<(), ApiError> {
        if self.max_versions_per_package > 0 && existing_versions >= self.max_versions_per_package {
            return Err((
                StatusCode::FORBIDDEN,
                format!("This package already has the maximum of {} versions", self.max_versions_per_package),
            ));
        }
        Ok(())
    }
}

pub async fn load_policy(state: &AppState) -> PackagePolicy {
    let settings = load_settings(state, "packages.").await;
    let flag = |key: &str, fallback: bool| settings.get(key).map(|value| value == "true").unwrap_or(fallback);
    let number = |key: &str| settings.get(key).and_then(|value| value.parse::<i64>().ok()).unwrap_or(0).max(0);

    PackagePolicy {
        publishing_enabled: flag(PUBLISHING_ENABLED_KEY, true),
        admins_only: flag(ADMINS_ONLY_KEY, false),
        max_package_size_bytes: number(MAX_SIZE_KEY),
        max_versions_per_package: number(MAX_VERSIONS_KEY),
        allow_owner_delete: flag(OWNER_DELETE_KEY, true),
        allow_private_packages: flag(PRIVATE_PACKAGES_KEY, true),
        default_visibility: settings
            .get(DEFAULT_VISIBILITY_KEY)
            .and_then(|value| parse_visibility(value).ok())
            .unwrap_or(PUBLIC)
            .to_string(),
    }
}

pub async fn is_admin(state: &AppState, user_id: &str) -> Result<bool, ApiError> {
    Ok(sqlx::query_as::<_, (i64,)>(sql("SELECT is_admin FROM users WHERE id = $1"))
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(|(admin,)| admin != 0)
        .unwrap_or(false))
}

pub async fn own_policy(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<UserPackagePolicy>, ApiError> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;
    let user_is_admin = is_admin(&state, &user_id).await?;
    let policy = load_policy(&state).await;

    Ok(Json(UserPackagePolicy {
        can_publish: policy.check_can_publish(user_is_admin).is_ok(),
        can_delete_own: user_is_admin || policy.allow_owner_delete,
        can_publish_private: user_is_admin || policy.allow_private_packages,
        policy,
    }))
}

pub async fn get_policy(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<PackagePolicy>, ApiError> {
    require_admin(&state, &jar).await?;
    Ok(Json(load_policy(&state).await))
}

pub async fn update_policy(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: crate::accounts::client_info::ClientInfo,
    Json(payload): Json<PackagePolicy>,
) -> Result<Json<PackagePolicy>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;

    if payload.max_package_size_bytes < 0 || payload.max_versions_per_package < 0 {
        return Err((StatusCode::BAD_REQUEST, "Limits cannot be negative".to_string()));
    }
    let default_visibility = parse_visibility(&payload.default_visibility)?;

    let entries = [
        (PUBLISHING_ENABLED_KEY, payload.publishing_enabled.to_string()),
        (ADMINS_ONLY_KEY, payload.admins_only.to_string()),
        (MAX_SIZE_KEY, payload.max_package_size_bytes.to_string()),
        (MAX_VERSIONS_KEY, payload.max_versions_per_package.to_string()),
        (OWNER_DELETE_KEY, payload.allow_owner_delete.to_string()),
        (PRIVATE_PACKAGES_KEY, payload.allow_private_packages.to_string()),
        (DEFAULT_VISIBILITY_KEY, default_visibility.to_string()),
    ];
    save_settings(&state, &entries)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::admin::audit::record(&state, &admin_id, &client, "settings.packages", "", &format!("publishing {}, admins only {}, private packages {}, default {}", payload.publishing_enabled, payload.admins_only, payload.allow_private_packages, default_visibility)).await;

    Ok(Json(load_policy(&state).await))
}

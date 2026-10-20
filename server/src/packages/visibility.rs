use std::collections::HashMap;

use axum::http::StatusCode;

use crate::db::sql;
use crate::models::Package;
use crate::AppState;

type ApiError = (StatusCode, String);

fn internal_error(error: impl ToString) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

pub async fn find_by_name(state: &AppState, name: &str) -> Result<Option<Package>, ApiError> {
    sqlx::query_as::<_, Package>(sql(
        "SELECT id, owner_id, namespace, name, description, visibility, created_at FROM packages \
         WHERE namespace = 'typstdrive' AND name = $1",
    ))
    .bind(name)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)
}

pub async fn list_all(state: &AppState) -> Result<Vec<Package>, ApiError> {
    sqlx::query_as::<_, Package>(sql(
        "SELECT p.id, p.owner_id, p.namespace, p.name, p.description, p.visibility, p.created_at, \
         u.username as owner_name, \
         (SELECT v.version FROM package_versions v WHERE v.package_id = p.id ORDER BY v.created_at DESC LIMIT 1) as latest_version \
         FROM packages p JOIN users u ON u.id = p.owner_id \
         ORDER BY p.name ASC",
    ))
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)
}

pub async fn list_visible(state: &AppState, account_id: &str) -> Result<Vec<Package>, ApiError> {
    sqlx::query_as::<_, Package>(sql(
        "SELECT p.id, p.owner_id, p.namespace, p.name, p.description, p.visibility, p.created_at, \
         u.username as owner_name, \
         (SELECT v.version FROM package_versions v WHERE v.package_id = p.id ORDER BY v.created_at DESC LIMIT 1) as latest_version \
         FROM packages p JOIN users u ON u.id = p.owner_id \
         WHERE (p.visibility = 'public' OR p.owner_id = $1 \
         OR EXISTS (SELECT 1 FROM package_members m WHERE m.package_id = p.id AND m.user_id = $2) \
         OR EXISTS (SELECT 1 FROM package_teams t WHERE t.package_id = p.id AND (t.team_id = $3 \
         OR t.team_id IN (SELECT tm.team_id FROM team_members tm WHERE tm.user_id = $4)))) \
         ORDER BY p.name ASC",
    ))
    .bind(account_id)
    .bind(account_id)
    .bind(account_id)
    .bind(account_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)
}

pub async fn can_view(state: &AppState, package_id: &str, account_id: &str) -> bool {
    sqlx::query_as::<_, (String,)>(sql(
        "SELECT p.id FROM packages p WHERE p.id = $1 \
         AND (p.visibility = 'public' OR p.owner_id = $2 \
         OR EXISTS (SELECT 1 FROM package_members m WHERE m.package_id = p.id AND m.user_id = $3) \
         OR EXISTS (SELECT 1 FROM package_teams t WHERE t.package_id = p.id AND (t.team_id = $4 \
         OR t.team_id IN (SELECT tm.team_id FROM team_members tm WHERE tm.user_id = $5))))",
    ))
    .bind(package_id)
    .bind(account_id)
    .bind(account_id)
    .bind(account_id)
    .bind(account_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .is_some()
}

pub async fn load_files_for(state: &AppState, account_id: &str) -> HashMap<String, HashMap<String, Vec<u8>>> {
    let rows = sqlx::query_as::<_, (String, String, String, Vec<u8>)>(sql(
        "SELECT p.name, v.version, f.path, f.data \
         FROM package_files f \
         JOIN package_versions v ON v.id = f.version_id \
         JOIN packages p ON p.id = v.package_id \
         WHERE (p.visibility = 'public' OR p.owner_id = $1 \
         OR EXISTS (SELECT 1 FROM package_members m WHERE m.package_id = p.id AND m.user_id = $2) \
         OR EXISTS (SELECT 1 FROM package_teams t WHERE t.package_id = p.id AND (t.team_id = $3 \
         OR t.team_id IN (SELECT tm.team_id FROM team_members tm WHERE tm.user_id = $4))))",
    ))
    .bind(account_id)
    .bind(account_id)
    .bind(account_id)
    .bind(account_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut packages: HashMap<String, HashMap<String, Vec<u8>>> = HashMap::new();
    for (name, version, path, data) in rows {
        packages.entry(format!("{name}:{version}")).or_default().insert(path, data);
    }
    packages
}

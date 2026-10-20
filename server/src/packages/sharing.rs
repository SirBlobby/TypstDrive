use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::accounts::lookup::shareable_user_id;
use crate::accounts::sessions::CurrentSession;
use crate::db::sql;
use crate::models::Package;
use crate::packages::policy::{is_admin, load_policy, parse_visibility};
use crate::packages::visibility::find_by_name;
use crate::AppState;

type ApiError = (StatusCode, String);

#[derive(Serialize, FromRow)]
pub struct SharedUser {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
}

#[derive(Serialize, FromRow)]
pub struct SharedTeam {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct PackageAccess {
    pub visibility: String,
    pub users: Vec<SharedUser>,
    pub teams: Vec<SharedTeam>,
}

#[derive(Deserialize)]
pub struct VisibilityRequest {
    pub visibility: String,
}

#[derive(Deserialize)]
pub struct AddUserRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct AddTeamRequest {
    pub team_id: String,
}

struct ManagedPackage {
    package: Package,
    signed_in_user_id: String,
    is_admin: bool,
}

fn internal_error(error: impl ToString) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

async fn managed_package(
    state: &AppState,
    jar: &SignedCookieJar,
    session: &CurrentSession,
    name: &str,
) -> Result<ManagedPackage, ApiError> {
    let acting_account_id = jar
        .get("session_user_id")
        .map(|cookie| cookie.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;
    let package = find_by_name(state, name)
        .await?
        .ok_or((StatusCode::NOT_FOUND, "Package not found".to_string()))?;
    let signed_in_user_id = session.0.user_id.clone();
    let user_is_admin = is_admin(state, &signed_in_user_id).await?;

    if package.owner_id != acting_account_id && !user_is_admin {
        return Err((StatusCode::FORBIDDEN, "Only the package owner can change who can use it".to_string()));
    }
    Ok(ManagedPackage { package, signed_in_user_id, is_admin: user_is_admin })
}

async fn access_for(state: &AppState, package_id: &str) -> Result<Json<PackageAccess>, ApiError> {
    let (visibility,) = sqlx::query_as::<_, (String,)>(sql("SELECT visibility FROM packages WHERE id = $1"))
        .bind(package_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;

    let users = sqlx::query_as::<_, SharedUser>(sql(
        "SELECT u.id, u.username, u.email FROM package_members m JOIN users u ON u.id = m.user_id \
         WHERE m.package_id = $1 ORDER BY u.username ASC",
    ))
    .bind(package_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let teams = sqlx::query_as::<_, SharedTeam>(sql(
        "SELECT t.id, t.name FROM package_teams pt JOIN teams t ON t.id = pt.team_id \
         WHERE pt.package_id = $1 ORDER BY t.name ASC",
    ))
    .bind(package_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(Json(PackageAccess { visibility, users, teams }))
}

pub async fn get_access(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    session: CurrentSession,
    Path(name): Path<String>,
) -> Result<Json<PackageAccess>, ApiError> {
    let managed = managed_package(&state, &jar, &session, &name).await?;
    access_for(&state, &managed.package.id).await
}

pub async fn update_visibility(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    session: CurrentSession,
    Path(name): Path<String>,
    Json(payload): Json<VisibilityRequest>,
) -> Result<Json<PackageAccess>, ApiError> {
    let managed = managed_package(&state, &jar, &session, &name).await?;
    let visibility = parse_visibility(&payload.visibility)?;
    load_policy(&state).await.check_visibility(visibility, managed.is_admin)?;

    sqlx::query(sql("UPDATE packages SET visibility = $1 WHERE id = $2"))
        .bind(visibility)
        .bind(&managed.package.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    access_for(&state, &managed.package.id).await
}

pub async fn add_user(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    session: CurrentSession,
    Path(name): Path<String>,
    Json(payload): Json<AddUserRequest>,
) -> Result<Json<PackageAccess>, ApiError> {
    let managed = managed_package(&state, &jar, &session, &name).await?;
    let member_id = shareable_user_id(&state, payload.email.trim()).await?;
    if member_id == managed.package.owner_id {
        return Err((StatusCode::BAD_REQUEST, "The owner can always use their own package".to_string()));
    }

    remove_member(&state, &managed.package.id, &member_id).await?;
    sqlx::query(sql("INSERT INTO package_members (package_id, user_id) VALUES ($1, $2)"))
        .bind(&managed.package.id)
        .bind(&member_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    access_for(&state, &managed.package.id).await
}

async fn remove_member(state: &AppState, package_id: &str, user_id: &str) -> Result<(), ApiError> {
    sqlx::query(sql("DELETE FROM package_members WHERE package_id = $1 AND user_id = $2"))
        .bind(package_id)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(())
}

pub async fn remove_user(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    session: CurrentSession,
    Path((name, user_id)): Path<(String, String)>,
) -> Result<Json<PackageAccess>, ApiError> {
    let managed = managed_package(&state, &jar, &session, &name).await?;
    remove_member(&state, &managed.package.id, &user_id).await?;
    access_for(&state, &managed.package.id).await
}

async fn remove_team_access(state: &AppState, package_id: &str, team_id: &str) -> Result<(), ApiError> {
    sqlx::query(sql("DELETE FROM package_teams WHERE package_id = $1 AND team_id = $2"))
        .bind(package_id)
        .bind(team_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(())
}

pub async fn add_team(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    session: CurrentSession,
    Path(name): Path<String>,
    Json(payload): Json<AddTeamRequest>,
) -> Result<Json<PackageAccess>, ApiError> {
    let managed = managed_package(&state, &jar, &session, &name).await?;
    if !managed.is_admin {
        crate::teams::require_role(&state, &payload.team_id, &managed.signed_in_user_id).await?;
    }

    remove_team_access(&state, &managed.package.id, &payload.team_id).await?;
    sqlx::query(sql("INSERT INTO package_teams (package_id, team_id) VALUES ($1, $2)"))
        .bind(&managed.package.id)
        .bind(&payload.team_id)
        .execute(&state.db)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Team not found".to_string()))?;
    access_for(&state, &managed.package.id).await
}

pub async fn remove_team(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    session: CurrentSession,
    Path((name, team_id)): Path<(String, String)>,
) -> Result<Json<PackageAccess>, ApiError> {
    let managed = managed_package(&state, &jar, &session, &name).await?;
    remove_team_access(&state, &managed.package.id, &team_id).await?;
    access_for(&state, &managed.package.id).await
}

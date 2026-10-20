use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::{policy, require_role, ApiError, TeamRole};
use crate::accounts::passwords;
use crate::db::{sql, timestamp_now};
use crate::AppState;

const MAX_NAME_LENGTH: usize = 80;

#[derive(Serialize, FromRow)]
pub struct TeamSummary {
    pub id: String,
    pub name: String,
    pub role: String,
    pub member_count: i64,
    pub created_at: String,
}

#[derive(Serialize, FromRow)]
pub struct TeamMemberView {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct TeamDetail {
    #[serde(flatten)]
    pub summary: TeamSummary,
    pub members: Vec<TeamMemberView>,
    pub max_members: i64,
}

#[derive(Deserialize)]
pub struct TeamNameRequest {
    pub name: String,
}

pub fn session_user(jar: &SignedCookieJar) -> Result<String, ApiError> {
    jar.get("session_user_id")
        .map(|cookie| cookie.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))
}

pub fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn clean_name(name: &str) -> Result<String, ApiError> {
    let name = name.trim();
    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Team name cannot be empty".to_string()));
    }
    if name.chars().count() > MAX_NAME_LENGTH {
        return Err((StatusCode::BAD_REQUEST, format!("Team name can be at most {MAX_NAME_LENGTH} characters")));
    }
    Ok(name.to_string())
}

async fn account_name_taken(state: &AppState, username: &str, except_id: &str) -> bool {
    sqlx::query_as::<_, (String,)>(sql("SELECT id FROM users WHERE LOWER(username) = LOWER($1) AND id <> $2"))
        .bind(username)
        .bind(except_id)
        .fetch_optional(&state.db)
        .await
        .map(|row| row.is_some())
        .unwrap_or(true)
}

async fn account_name_for(state: &AppState, team_name: &str, team_id: &str) -> String {
    if !account_name_taken(state, team_name, team_id).await {
        return team_name.to_string();
    }
    for number in 2..=50 {
        let candidate = format!("{team_name} ({number})");
        if !account_name_taken(state, &candidate, team_id).await {
            return candidate;
        }
    }
    format!("{team_name} ({})", &team_id[..8])
}

pub async fn team_summary(state: &AppState, team_id: &str, user_id: &str) -> Result<TeamSummary, ApiError> {
    sqlx::query_as::<_, TeamSummary>(sql(
        "SELECT t.id, t.name, m.role, \
         CAST((SELECT COUNT(*) FROM team_members c WHERE c.team_id = t.id) AS BIGINT) AS member_count, t.created_at \
         FROM teams t INNER JOIN team_members m ON m.team_id = t.id AND m.user_id = $1 WHERE t.id = $2",
    ))
    .bind(user_id)
    .bind(team_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or((StatusCode::NOT_FOUND, "Team not found".to_string()))
}

pub async fn team_members(state: &AppState, team_id: &str) -> Result<Vec<TeamMemberView>, ApiError> {
    sqlx::query_as::<_, TeamMemberView>(sql(
        "SELECT m.user_id, u.username, COALESCE(u.email, '') AS email, m.role, m.created_at \
         FROM team_members m INNER JOIN users u ON u.id = m.user_id WHERE m.team_id = $1 \
         ORDER BY CASE m.role WHEN 'owner' THEN 0 WHEN 'admin' THEN 1 WHEN 'member' THEN 2 ELSE 3 END, u.username ASC",
    ))
    .bind(team_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)
}

pub async fn user_teams(state: &AppState, user_id: &str) -> Result<Vec<TeamSummary>, sqlx::Error> {
    if !policy::load_policy(state).await.enabled {
        return Ok(Vec::new());
    }
    sqlx::query_as::<_, TeamSummary>(sql(
        "SELECT t.id, t.name, m.role, \
         CAST((SELECT COUNT(*) FROM team_members c WHERE c.team_id = t.id) AS BIGINT) AS member_count, t.created_at \
         FROM teams t INNER JOIN team_members m ON m.team_id = t.id WHERE m.user_id = $1 ORDER BY t.name ASC",
    ))
    .bind(user_id)
    .fetch_all(&state.db)
    .await
}

pub async fn list_teams(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<Vec<TeamSummary>>, ApiError> {
    let user_id = session_user(&jar)?;
    Ok(Json(user_teams(&state, &user_id).await.map_err(internal_error)?))
}

pub async fn create_team(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<TeamNameRequest>,
) -> Result<(StatusCode, Json<TeamSummary>), ApiError> {
    let user_id = session_user(&jar)?;
    let name = clean_name(&payload.name)?;
    let team_policy = policy::load_policy(&state).await;
    policy::check_can_create(&state, &team_policy, &user_id).await?;

    let team_id = Uuid::new_v4().to_string();
    let account_name = account_name_for(&state, &name, &team_id).await;
    let now = timestamp_now();

    let mut transaction = state.db.begin().await.map_err(internal_error)?;
    sqlx::query(sql(
        "INSERT INTO users (id, username, email, password_hash, is_admin, email_verified, account_kind, created_at) \
         VALUES ($1, $2, NULL, $3, 0, 0, 'team', $4)",
    ))
    .bind(&team_id)
    .bind(&account_name)
    .bind(passwords::NO_PASSWORD)
    .bind(&now)
    .execute(&mut *transaction)
    .await
    .map_err(internal_error)?;
    sqlx::query(sql("INSERT INTO teams (id, name, created_by, created_at) VALUES ($1, $2, $3, $4)"))
        .bind(&team_id)
        .bind(&name)
        .bind(&user_id)
        .bind(&now)
        .execute(&mut *transaction)
        .await
        .map_err(internal_error)?;
    sqlx::query(sql("INSERT INTO team_members (team_id, user_id, role, created_at) VALUES ($1, $2, $3, $4)"))
        .bind(&team_id)
        .bind(&user_id)
        .bind(TeamRole::Owner.as_str())
        .bind(&now)
        .execute(&mut *transaction)
        .await
        .map_err(internal_error)?;
    transaction.commit().await.map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(team_summary(&state, &team_id, &user_id).await?)))
}

pub async fn get_team(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(team_id): Path<String>,
) -> Result<Json<TeamDetail>, ApiError> {
    let user_id = session_user(&jar)?;
    let summary = team_summary(&state, &team_id, &user_id).await?;
    let members = team_members(&state, &team_id).await?;
    let max_members = policy::load_policy(&state).await.max_members_per_team;
    Ok(Json(TeamDetail { summary, members, max_members }))
}

pub async fn rename_team(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(team_id): Path<String>,
    Json(payload): Json<TeamNameRequest>,
) -> Result<Json<TeamSummary>, ApiError> {
    let user_id = session_user(&jar)?;
    let role = require_role(&state, &team_id, &user_id).await?;
    if !role.can_manage_members() {
        return Err((StatusCode::FORBIDDEN, "Only the owner and admins can rename the team".to_string()));
    }

    let name = clean_name(&payload.name)?;
    let account_name = account_name_for(&state, &name, &team_id).await;
    sqlx::query(sql("UPDATE teams SET name = $1 WHERE id = $2"))
        .bind(&name)
        .bind(&team_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    sqlx::query(sql("UPDATE users SET username = $1 WHERE id = $2 AND account_kind = 'team'"))
        .bind(&account_name)
        .bind(&team_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    Ok(Json(team_summary(&state, &team_id, &user_id).await?))
}

pub async fn delete_team(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(team_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let user_id = session_user(&jar)?;
    if require_role(&state, &team_id, &user_id).await? != TeamRole::Owner {
        return Err((StatusCode::FORBIDDEN, "Only the owner can delete the team".to_string()));
    }
    remove_team_account(&state, &team_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_team_account(state: &AppState, team_id: &str) -> Result<(), ApiError> {
    sqlx::query(sql("DELETE FROM users WHERE id = $1 AND account_kind = 'team'"))
        .bind(team_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    crate::sharing::access::revoke_user_sessions(state, team_id).await;
    Ok(())
}

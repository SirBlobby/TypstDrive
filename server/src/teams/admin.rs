use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Serialize;
use sqlx::FromRow;

use super::handlers::{internal_error, remove_team_account};
use super::ApiError;
use crate::accounts::client_info::ClientInfo;
use crate::admin::{audit, users::require_admin};
use crate::db::sql;
use crate::AppState;

#[derive(Serialize, FromRow)]
pub struct AdminTeamView {
    pub id: String,
    pub name: String,
    pub owner_name: Option<String>,
    pub member_count: i64,
    pub created_at: String,
}

pub async fn list_all_teams(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<Vec<AdminTeamView>>, ApiError> {
    require_admin(&state, &jar).await?;
    let teams = sqlx::query_as::<_, AdminTeamView>(sql(
        "SELECT t.id, t.name, \
         (SELECT u.username FROM team_members m INNER JOIN users u ON u.id = m.user_id WHERE m.team_id = t.id AND m.role = 'owner' LIMIT 1) AS owner_name, \
         CAST((SELECT COUNT(*) FROM team_members c WHERE c.team_id = t.id) AS BIGINT) AS member_count, t.created_at \
         FROM teams t ORDER BY t.name ASC",
    ))
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(Json(teams))
}

pub async fn delete_any_team(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
    Path(team_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    let name = sqlx::query_as::<_, (String,)>(sql("SELECT name FROM teams WHERE id = $1"))
        .bind(&team_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .map(|(name,)| name)
        .ok_or((StatusCode::NOT_FOUND, "Team not found".to_string()))?;

    remove_team_account(&state, &team_id).await?;
    audit::record(&state, &admin_id, &client, "team.delete", &name, "").await;
    Ok(StatusCode::NO_CONTENT)
}

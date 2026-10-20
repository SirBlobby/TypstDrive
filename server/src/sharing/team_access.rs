use serde::Serialize;
use sqlx::FromRow;

use crate::db::sql;
use crate::AppState;

#[derive(Serialize, FromRow)]
pub struct TeamMemberAccess {
    pub user_id: String,
    pub username: String,
    pub email: Option<String>,
    pub team_role: String,
}

#[derive(Serialize)]
pub struct TeamAccess {
    pub team_name: String,
    pub members: Vec<TeamMemberAccess>,
}

async fn team_name(state: &AppState, owner_id: &str) -> Option<String> {
    sqlx::query_as::<_, (String,)>(sql(
        "SELECT t.name FROM teams t INNER JOIN users u ON u.id = t.id WHERE t.id = $1 AND u.account_kind = 'team'",
    ))
    .bind(owner_id)
    .fetch_optional(&state.db)
    .await
    .ok()?
    .map(|(name,)| name)
}

pub async fn team_access_for(state: &AppState, owner_id: &str, viewer_id: &str) -> Option<TeamAccess> {
    let team_name = team_name(state, owner_id).await?;
    if owner_id != viewer_id && crate::teams::content_role(state, owner_id, viewer_id).await.is_none() {
        return None;
    }

    let members = sqlx::query_as::<_, TeamMemberAccess>(sql(
        "SELECT u.id AS user_id, u.username, u.email, tm.role AS team_role \
         FROM team_members tm INNER JOIN users u ON u.id = tm.user_id \
         WHERE tm.team_id = $1 ORDER BY u.username ASC",
    ))
    .bind(owner_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Some(TeamAccess { team_name, members })
}

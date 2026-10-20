pub mod admin;
pub mod handlers;
pub mod members;
pub mod policy;
pub mod workspace;

use axum::http::StatusCode;

use crate::db::sql;
use crate::sharing::access::AccessRole;
use crate::AppState;

type ApiError = (StatusCode, String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

impl TeamRole {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "owner" => Some(Self::Owner),
            "admin" => Some(Self::Admin),
            "member" => Some(Self::Member),
            "viewer" => Some(Self::Viewer),
            _ => None,
        }
    }

    pub fn parse_assignable(value: &str) -> Result<Self, ApiError> {
        match Self::parse(value.trim()) {
            Some(Self::Owner) | None => Err((StatusCode::BAD_REQUEST, "Role must be admin, member or viewer".to_string())),
            Some(role) => Ok(role),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Viewer => "viewer",
        }
    }

    pub fn can_manage_members(self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    pub fn can_change_content(self) -> bool {
        !matches!(self, Self::Viewer)
    }
}

pub async fn membership(state: &AppState, team_id: &str, user_id: &str) -> Option<TeamRole> {
    sqlx::query_as::<_, (String,)>(sql("SELECT role FROM team_members WHERE team_id = $1 AND user_id = $2"))
        .bind(team_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .and_then(|(role,)| TeamRole::parse(&role))
}

pub async fn content_role(state: &AppState, owner_id: &str, user_id: &str) -> Option<AccessRole> {
    if owner_id == user_id {
        return None;
    }
    membership(state, owner_id, user_id).await.map(|role| {
        if role.can_change_content() { AccessRole::Owner } else { AccessRole::Viewer }
    })
}

pub async fn require_role(state: &AppState, team_id: &str, user_id: &str) -> Result<TeamRole, ApiError> {
    membership(state, team_id, user_id)
        .await
        .ok_or((StatusCode::NOT_FOUND, "Team not found".to_string()))
}

pub async fn handle_departing_user(state: &AppState, user_id: &str) {
    let owned_teams = sqlx::query_as::<_, (String,)>(sql("SELECT team_id FROM team_members WHERE user_id = $1 AND role = 'owner'"))
        .bind(user_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    for (team_id,) in owned_teams {
        let successor = sqlx::query_as::<_, (String,)>(sql(
            "SELECT user_id FROM team_members WHERE team_id = $1 AND user_id <> $2 \
             ORDER BY CASE role WHEN 'admin' THEN 0 WHEN 'member' THEN 1 ELSE 2 END, created_at ASC LIMIT 1",
        ))
        .bind(&team_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

        let result = match successor {
            Some((successor_id,)) => sqlx::query(sql("UPDATE team_members SET role = 'owner' WHERE team_id = $1 AND user_id = $2"))
                .bind(&team_id)
                .bind(&successor_id)
                .execute(&state.db)
                .await,
            None => sqlx::query(sql("DELETE FROM users WHERE id = $1 AND account_kind = 'team'"))
                .bind(&team_id)
                .execute(&state.db)
                .await,
        };
        if let Err(error) = result {
            tracing::warn!("Could not hand over team {team_id}: {error}");
        }
    }
}

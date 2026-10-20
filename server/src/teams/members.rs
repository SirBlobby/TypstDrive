use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Deserialize;

use super::handlers::{internal_error, session_user, team_members, TeamMemberView};
use super::{policy, require_role, ApiError, TeamRole};
use crate::accounts::lookup;
use crate::db::{sql, timestamp_now};
use crate::AppState;

#[derive(Deserialize)]
pub struct AddMemberRequest {
    pub email: String,
    pub role: String,
}

#[derive(Deserialize)]
pub struct ChangeRoleRequest {
    pub role: String,
}

async fn require_manager(state: &AppState, team_id: &str, user_id: &str) -> Result<TeamRole, ApiError> {
    let role = require_role(state, team_id, user_id).await?;
    if role.can_manage_members() {
        Ok(role)
    } else {
        Err((StatusCode::FORBIDDEN, "Only the owner and admins can manage members".to_string()))
    }
}

fn check_authority(actor: TeamRole, target_current: TeamRole, target_new: Option<TeamRole>) -> Result<(), ApiError> {
    if target_current == TeamRole::Owner {
        return Err((StatusCode::FORBIDDEN, "The owner's role cannot be changed".to_string()));
    }
    let touches_admin = target_current == TeamRole::Admin || target_new == Some(TeamRole::Admin);
    if actor != TeamRole::Owner && touches_admin {
        return Err((StatusCode::FORBIDDEN, "Only the owner can add, change or remove admins".to_string()));
    }
    Ok(())
}

async fn send_invitation_email(state: &AppState, team_id: &str, inviter_id: &str, email: &str, role: TeamRole) {
    if !state.mailer.is_configured() {
        return;
    }
    let team_name = sqlx::query_as::<_, (String,)>(sql("SELECT name FROM teams WHERE id = $1"))
        .bind(team_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .map(|(name,)| name)
        .unwrap_or_default();
    let inviter_name = lookup::user_by_id(state, inviter_id)
        .await
        .ok()
        .flatten()
        .map(|user| user.username)
        .unwrap_or_default();
    let link = crate::site_url::link("/drive");
    let message = crate::email::messages::team_invitation(&team_name, &inviter_name, role.as_str(), link.as_deref());
    if let Err(error) = state.mailer.send(email, message).await {
        tracing::warn!("Team invitation email to {email} failed: {error}");
    }
}

pub async fn add_member(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(team_id): Path<String>,
    Json(payload): Json<AddMemberRequest>,
) -> Result<Json<Vec<TeamMemberView>>, ApiError> {
    let user_id = session_user(&jar)?;
    let actor_role = require_manager(&state, &team_id, &user_id).await?;
    let new_role = TeamRole::parse_assignable(&payload.role)?;
    check_authority(actor_role, TeamRole::Member, Some(new_role))?;

    let member_id = lookup::shareable_user_id(&state, &payload.email).await?;
    if super::membership(&state, &team_id, &member_id).await.is_some() {
        return Err((StatusCode::CONFLICT, "That person is already in the team".to_string()));
    }

    let max_members = policy::load_policy(&state).await.max_members_per_team;
    if max_members > 0 && team_members(&state, &team_id).await?.len() as i64 >= max_members {
        return Err((StatusCode::CONFLICT, format!("A team can have at most {max_members} members")));
    }

    sqlx::query(sql("INSERT INTO team_members (team_id, user_id, role, created_at) VALUES ($1, $2, $3, $4)"))
        .bind(&team_id)
        .bind(&member_id)
        .bind(new_role.as_str())
        .bind(timestamp_now())
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    send_invitation_email(&state, &team_id, &user_id, payload.email.trim(), new_role).await;
    Ok(Json(team_members(&state, &team_id).await?))
}

pub async fn change_member_role(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path((team_id, member_id)): Path<(String, String)>,
    Json(payload): Json<ChangeRoleRequest>,
) -> Result<Json<Vec<TeamMemberView>>, ApiError> {
    let user_id = session_user(&jar)?;
    let actor_role = require_manager(&state, &team_id, &user_id).await?;
    let current_role = super::membership(&state, &team_id, &member_id)
        .await
        .ok_or((StatusCode::NOT_FOUND, "Member not found".to_string()))?;
    let new_role = TeamRole::parse_assignable(&payload.role)?;
    check_authority(actor_role, current_role, Some(new_role))?;

    sqlx::query(sql("UPDATE team_members SET role = $1 WHERE team_id = $2 AND user_id = $3"))
        .bind(new_role.as_str())
        .bind(&team_id)
        .bind(&member_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    crate::sharing::access::revoke_user_sessions(&state, &member_id).await;
    Ok(Json(team_members(&state, &team_id).await?))
}

pub async fn remove_member(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path((team_id, member_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let user_id = session_user(&jar)?;
    let current_role = super::membership(&state, &team_id, &member_id)
        .await
        .ok_or((StatusCode::NOT_FOUND, "Member not found".to_string()))?;

    if member_id == user_id {
        if current_role == TeamRole::Owner {
            return Err((StatusCode::BAD_REQUEST, "Transfer ownership before leaving the team".to_string()));
        }
    } else {
        let actor_role = require_manager(&state, &team_id, &user_id).await?;
        check_authority(actor_role, current_role, None)?;
    }

    sqlx::query(sql("DELETE FROM team_members WHERE team_id = $1 AND user_id = $2"))
        .bind(&team_id)
        .bind(&member_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    crate::sharing::access::revoke_user_sessions(&state, &member_id).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn transfer_ownership(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path((team_id, member_id)): Path<(String, String)>,
) -> Result<Json<Vec<TeamMemberView>>, ApiError> {
    let user_id = session_user(&jar)?;
    if require_role(&state, &team_id, &user_id).await? != TeamRole::Owner {
        return Err((StatusCode::FORBIDDEN, "Only the owner can hand over the team".to_string()));
    }
    if super::membership(&state, &team_id, &member_id).await.is_none() || member_id == user_id {
        return Err((StatusCode::BAD_REQUEST, "Choose another member of the team".to_string()));
    }

    let mut transaction = state.db.begin().await.map_err(internal_error)?;
    sqlx::query(sql("UPDATE team_members SET role = 'admin' WHERE team_id = $1 AND user_id = $2"))
        .bind(&team_id)
        .bind(&user_id)
        .execute(&mut *transaction)
        .await
        .map_err(internal_error)?;
    sqlx::query(sql("UPDATE team_members SET role = 'owner' WHERE team_id = $1 AND user_id = $2"))
        .bind(&team_id)
        .bind(&member_id)
        .execute(&mut *transaction)
        .await
        .map_err(internal_error)?;
    transaction.commit().await.map_err(internal_error)?;

    Ok(Json(team_members(&state, &team_id).await?))
}

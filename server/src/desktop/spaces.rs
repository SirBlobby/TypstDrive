use super::*;
use crate::teams::handlers::{internal_error, user_teams, TeamSummary};

const SPACE_HEADER: &str = "x-workspace";

fn requested_team(headers: &HeaderMap) -> Option<String> {
    headers
        .get(SPACE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && value != "personal")
}

async fn space_account(state: &AppState, headers: &HeaderMap, writing: bool) -> Result<String, DesktopError> {
    let user_id = authenticate(state, headers).await?;
    let Some(team_id) = requested_team(headers) else {
        return Ok(user_id);
    };

    if !crate::teams::policy::load_policy(state).await.enabled {
        return Err((StatusCode::FORBIDDEN, "Team spaces are turned off on this server".to_string()));
    }
    let role = crate::teams::membership(state, &team_id, &user_id)
        .await
        .ok_or((StatusCode::FORBIDDEN, "You are not a member of this team".to_string()))?;
    if writing && !role.can_change_content() {
        return Err((StatusCode::FORBIDDEN, "Viewers cannot change team content".to_string()));
    }
    Ok(team_id)
}

pub async fn readable_space(state: &AppState, headers: &HeaderMap) -> Result<String, DesktopError> {
    space_account(state, headers, false).await
}

pub async fn writable_space(state: &AppState, headers: &HeaderMap) -> Result<String, DesktopError> {
    space_account(state, headers, true).await
}

pub async fn list_spaces(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<TeamSummary>>, DesktopError> {
    let user_id = authenticate(&state, &headers).await?;
    Ok(Json(user_teams(&state, &user_id).await.map_err(internal_error)?))
}

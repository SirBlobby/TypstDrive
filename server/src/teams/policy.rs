use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};

use crate::accounts::client_info::ClientInfo;
use crate::admin::site_settings::{load_settings, save_settings};
use crate::admin::{audit, users::require_admin};
use crate::db::sql;
use crate::AppState;

const ENABLED_KEY: &str = "teams.enabled";
const CREATION_KEY: &str = "teams.creation";
const MAX_TEAMS_KEY: &str = "teams.max_teams_per_user";
const MAX_MEMBERS_KEY: &str = "teams.max_members_per_team";
const MAX_LIMIT: i64 = 10_000;

type ApiError = (StatusCode, String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPolicy {
    pub enabled: bool,
    pub creation: String,
    pub max_teams_per_user: i64,
    pub max_members_per_team: i64,
}

#[derive(Serialize)]
pub struct OwnTeamPolicy {
    #[serde(flatten)]
    pub policy: TeamPolicy,
    pub can_create: bool,
    pub owned_teams: i64,
}

pub async fn load_policy(state: &AppState) -> TeamPolicy {
    let settings = load_settings(state, "teams.").await;
    let number = |key: &str, fallback: i64| settings.get(key).and_then(|value| value.parse().ok()).unwrap_or(fallback);
    TeamPolicy {
        enabled: settings.get(ENABLED_KEY).map(|value| value == "true").unwrap_or(true),
        creation: settings
            .get(CREATION_KEY)
            .filter(|value| value.as_str() == "admins")
            .cloned()
            .unwrap_or_else(|| "everyone".to_string()),
        max_teams_per_user: number(MAX_TEAMS_KEY, 5),
        max_members_per_team: number(MAX_MEMBERS_KEY, 50),
    }
}

pub async fn owned_team_count(state: &AppState, user_id: &str) -> i64 {
    sqlx::query_as::<_, (i64,)>(sql("SELECT CAST(COUNT(*) AS BIGINT) FROM team_members WHERE user_id = $1 AND role = 'owner'"))
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map(|(count,)| count)
        .unwrap_or(0)
}

pub async fn is_admin(state: &AppState, user_id: &str) -> bool {
    sqlx::query_as::<_, (i64,)>(sql("SELECT is_admin FROM users WHERE id = $1"))
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .is_some_and(|(is_admin,)| is_admin != 0)
}

pub async fn check_can_create(state: &AppState, policy: &TeamPolicy, user_id: &str) -> Result<i64, ApiError> {
    if !policy.enabled {
        return Err((StatusCode::FORBIDDEN, "Team spaces are turned off on this server".to_string()));
    }
    let user_is_admin = is_admin(state, user_id).await;
    if policy.creation == "admins" && !user_is_admin {
        return Err((StatusCode::FORBIDDEN, "Only administrators can create team spaces".to_string()));
    }
    let owned = owned_team_count(state, user_id).await;
    if !user_is_admin && policy.max_teams_per_user > 0 && owned >= policy.max_teams_per_user {
        return Err((
            StatusCode::CONFLICT,
            format!("You can own at most {} team space(s)", policy.max_teams_per_user),
        ));
    }
    Ok(owned)
}

pub async fn own_policy(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<OwnTeamPolicy>, ApiError> {
    let user_id = jar
        .get("session_user_id")
        .map(|cookie| cookie.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;
    let policy = load_policy(&state).await;
    let can_create = check_can_create(&state, &policy, &user_id).await.is_ok();
    let owned_teams = owned_team_count(&state, &user_id).await;
    Ok(Json(OwnTeamPolicy { policy, can_create, owned_teams }))
}

pub async fn get_policy(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<TeamPolicy>, ApiError> {
    require_admin(&state, &jar).await?;
    Ok(Json(load_policy(&state).await))
}

pub async fn update_policy(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
    Json(payload): Json<TeamPolicy>,
) -> Result<Json<TeamPolicy>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;

    if payload.creation != "everyone" && payload.creation != "admins" {
        return Err((StatusCode::BAD_REQUEST, "Creation must be everyone or admins".to_string()));
    }
    for (name, value) in [("Teams per user", payload.max_teams_per_user), ("Members per team", payload.max_members_per_team)] {
        if !(0..=MAX_LIMIT).contains(&value) {
            return Err((StatusCode::BAD_REQUEST, format!("{name} must be between 0 and {MAX_LIMIT}")));
        }
    }

    let entries = [
        (ENABLED_KEY, payload.enabled.to_string()),
        (CREATION_KEY, payload.creation.clone()),
        (MAX_TEAMS_KEY, payload.max_teams_per_user.to_string()),
        (MAX_MEMBERS_KEY, payload.max_members_per_team.to_string()),
    ];
    save_settings(&state, &entries)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    let details = format!(
        "enabled {}, creation {}, teams per user {}, members per team {}",
        payload.enabled, payload.creation, payload.max_teams_per_user, payload.max_members_per_team
    );
    audit::record(&state, &admin_id, &client, "settings.teams", "", &details).await;
    Ok(Json(load_policy(&state).await))
}

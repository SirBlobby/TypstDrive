use axum::http::{Method, StatusCode, Uri};
use axum::http::HeaderMap;

use super::TeamRole;
use crate::AppState;

const WORKSPACE_HEADER: &str = "x-workspace";
const WORKSPACE_QUERY: &str = "workspace=";

const TEAM_SCOPED_PREFIXES: &[&str] = &[
    "/api/docs",
    "/api/folders",
    "/api/files",
    "/api/fonts",
    "/api/projects",
    "/api/packages",
    "/api/compile",
    "/api/export",
    "/api/import",
    "/api/share-info",
    "/api/item-info",
    "/api/share-links",
    "/api/pdf-links",
    "/api/auth/storage",
];

const PERSONAL_SEGMENTS: &[&str] = &["/comments", "/versions", "/suggestions"];

fn requested_team(headers: &HeaderMap, uri: &Uri) -> Option<String> {
    let from_header = headers
        .get(WORKSPACE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let from_query = uri
        .query()
        .and_then(|query| query.split('&').find_map(|pair| pair.strip_prefix(WORKSPACE_QUERY)))
        .map(str::to_string);
    from_header
        .or(from_query)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && value != "personal")
}

fn is_team_scoped(path: &str) -> bool {
    TEAM_SCOPED_PREFIXES.iter().any(|prefix| path == *prefix || path.starts_with(&format!("{prefix}/")))
        && !PERSONAL_SEGMENTS.iter().any(|segment| path.contains(segment))
        && !path.starts_with("/api/comments")
}

fn is_read_only(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

pub async fn acting_account(
    state: &AppState,
    method: &Method,
    uri: &Uri,
    headers: &HeaderMap,
    user_id: &str,
) -> Result<String, (StatusCode, String)> {
    let Some(team_id) = requested_team(headers, uri) else {
        return Ok(user_id.to_string());
    };
    let path = uri.path();
    let read_only_compile = path == "/api/compile" || path.starts_with("/api/export");
    if !is_team_scoped(path) {
        return Ok(user_id.to_string());
    }

    if !super::policy::load_policy(state).await.enabled {
        return Err((StatusCode::FORBIDDEN, "Team spaces are turned off on this server".to_string()));
    }
    let role = super::membership(state, &team_id, user_id)
        .await
        .ok_or((StatusCode::FORBIDDEN, "You are not a member of this team".to_string()))?;

    if role == TeamRole::Viewer && !is_read_only(method) && !read_only_compile {
        return Err((StatusCode::FORBIDDEN, "Viewers cannot change team content".to_string()));
    }
    Ok(team_id)
}

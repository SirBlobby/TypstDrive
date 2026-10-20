use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use axum::Json;

use super::database_error;
use crate::api::authentication::{authenticate, ApiScope};
use crate::teams::handlers::{user_teams, TeamSummary};
use crate::AppState;

pub async fn list_teams(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<TeamSummary>>, Response> {
    let caller = authenticate(&state, &headers, ApiScope::Read).await?;
    user_teams(&state, &caller.user_id).await.map(Json).map_err(database_error)
}

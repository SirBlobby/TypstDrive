use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Serialize;

use crate::accounts::client_info::ClientInfo;
use crate::admin::{audit, users::require_secure_admin};
use crate::db::sql;
use crate::AppState;

type ApiError = (StatusCode, String);

#[derive(Serialize)]
pub struct IssuedPassword {
    temporary_password: String,
    expires_at: String,
}

async fn other_username(state: &AppState, requester_id: &str, user_id: &str) -> Result<String, ApiError> {
    if requester_id == user_id {
        return Err((StatusCode::BAD_REQUEST, "Change your own sign-in settings from your account settings".to_string()));
    }
    sqlx::query_as::<_, (String,)>(sql("SELECT username FROM users WHERE id = $1 AND account_kind = 'user'"))
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(crate::server_errors::hidden)?
        .map(|(username,)| username)
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))
}

pub async fn issue_temporary_password(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
    Path(user_id): Path<String>,
) -> Result<Json<IssuedPassword>, ApiError> {
    let requester_id = require_secure_admin(&state, &jar).await?;
    let username = other_username(&state, &requester_id, &user_id).await?;

    let temporary = crate::accounts::temporary_password::assign(&state, &user_id).await?;
    crate::accounts::sessions::end_all_sessions(&state, &user_id, None).await?;
    crate::sharing::access::revoke_user_sessions(&state, &user_id).await;
    audit::record(&state, &requester_id, &client, "user.temporary_password", &username, "").await;

    Ok(Json(IssuedPassword { temporary_password: temporary.password, expires_at: temporary.expires_at }))
}

pub async fn reset_two_factor(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
    Path(user_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let requester_id = require_secure_admin(&state, &jar).await?;
    let username = other_username(&state, &requester_id, &user_id).await?;

    crate::accounts::two_factor::turn_off(&state, &user_id).await?;
    audit::record(&state, &requester_id, &client, "user.two_factor_reset", &username, "").await;
    Ok(StatusCode::NO_CONTENT)
}

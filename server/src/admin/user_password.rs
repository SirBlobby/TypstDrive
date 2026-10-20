use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Deserialize;

use crate::accounts::client_info::ClientInfo;
use crate::admin::{audit, users::require_admin};
use crate::db::sql;
use crate::AppState;

#[derive(Deserialize)]
pub struct SetPasswordRequest {
    pub password: String,
}

pub async fn set_user_password(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
    Path(user_id): Path<String>,
    Json(payload): Json<SetPasswordRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let requester_id = require_admin(&state, &jar).await?;
    if payload.password.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Password cannot be empty".to_string()));
    }

    let (username,) = sqlx::query_as::<_, (String,)>(sql("SELECT username FROM users WHERE id = $1 AND account_kind = 'user'"))
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let password_hash = crate::accounts::passwords::hash_password(&payload.password)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))?;
    sqlx::query(sql("UPDATE users SET password_hash = $1 WHERE id = $2"))
        .bind(&password_hash)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    if requester_id != user_id {
        crate::accounts::sessions::end_all_sessions(&state, &user_id, None).await?;
        crate::sharing::access::revoke_user_sessions(&state, &user_id).await;
    }
    audit::record(&state, &requester_id, &client, "user.set_password", &username, "").await;

    Ok(StatusCode::OK)
}

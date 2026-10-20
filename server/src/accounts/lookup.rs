use axum::http::StatusCode;

use crate::db::sql;
use crate::models::User;
use crate::AppState;

pub async fn user_by_email(state: &AppState, email: &str) -> Result<Option<User>, (StatusCode, String)> {
    sqlx::query_as::<_, User>(sql(
        "SELECT id, username, email, password_hash, is_admin, email_verified FROM users \
         WHERE LOWER(email) = LOWER($1) AND account_kind = 'user'",
    ))
    .bind(email.trim())
    .fetch_optional(&state.db)
    .await
    .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
}

pub async fn user_by_id(state: &AppState, user_id: &str) -> Result<Option<User>, (StatusCode, String)> {
    sqlx::query_as::<_, User>(sql(
        "SELECT id, username, email, password_hash, is_admin, email_verified FROM users \
         WHERE id = $1 AND account_kind = 'user'",
    ))
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
}

pub async fn shareable_user_id(state: &AppState, email: &str) -> Result<String, (StatusCode, String)> {
    let not_found = (StatusCode::NOT_FOUND, "No account uses that email address".to_string());
    let user = user_by_email(state, email).await?.ok_or(not_found)?;

    let settings = crate::accounts::auth_settings::load(state).await;
    if crate::accounts::auth_settings::verification_required(state, &settings) && user.email_verified == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            "That account has not verified its email address yet".to_string(),
        ));
    }
    Ok(user.id)
}

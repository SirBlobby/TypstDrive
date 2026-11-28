use axum::http::StatusCode;
use chrono::{TimeDelta, Utc};

use crate::accounts::passwords::{self, NO_PASSWORD};
use crate::accounts::random_text::{grouped, random_text};
use crate::accounts::sessions::format_time;
use crate::db::sql;
use crate::models::User;
use crate::AppState;

type ApiError = (StatusCode, String);

pub const LIFETIME_HOURS: i64 = 72;
const PASSWORD_LENGTH: usize = 16;
const GROUP_LENGTH: usize = 4;

pub struct TemporaryPassword {
    pub password: String,
    pub expires_at: String,
}

pub async fn assign(state: &AppState, user_id: &str) -> Result<TemporaryPassword, ApiError> {
    let password = grouped(&random_text(PASSWORD_LENGTH), GROUP_LENGTH);
    let password_hash = passwords::hash_password(&password)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))?;
    let expires_at = format_time(Utc::now() + TimeDelta::hours(LIFETIME_HOURS));
    sqlx::query(sql(
        "UPDATE users SET password_hash = $1, password_change_required = 1, temporary_password_expires_at = $2 WHERE id = $3",
    ))
    .bind(&password_hash)
    .bind(&expires_at)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(crate::server_errors::hidden)?;
    Ok(TemporaryPassword { password, expires_at })
}

pub async fn redeem(state: &AppState, user: &User) -> Result<(), ApiError> {
    let (expires_at,) = sqlx::query_as::<_, (Option<String>,)>(sql("SELECT temporary_password_expires_at FROM users WHERE id = $1"))
        .bind(&user.id)
        .fetch_one(&state.db)
        .await
        .map_err(crate::server_errors::hidden)?;
    if expires_at.is_some_and(|expires_at| expires_at < format_time(Utc::now())) {
        return Err((
            StatusCode::UNAUTHORIZED,
            "This temporary password has expired. Ask an administrator for a new one.".to_string(),
        ));
    }

    let used = sqlx::query(sql(
        "UPDATE users SET password_hash = $1 WHERE id = $2 AND password_hash = $3 AND password_change_required = 1",
    ))
    .bind(NO_PASSWORD)
    .bind(&user.id)
    .bind(&user.password_hash)
    .execute(&state.db)
    .await
    .map_err(crate::server_errors::hidden)?;
    if used.rows_affected() == 0 {
        return Err((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()));
    }
    Ok(())
}

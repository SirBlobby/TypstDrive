use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::CookieJar;
use uuid::Uuid;


use crate::{
    models::{SetupRequest, SetupStatus, User},
    AppState,
};
use crate::db::sql;

pub async fn setup_status(
    State(state): State<AppState>,
) -> Json<SetupStatus> {
    let count: (i64,) = sqlx::query_as(sql("SELECT COUNT(*) FROM users"))
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    Json(SetupStatus {
        needs_setup: count.0 == 0,
        registration_enabled: crate::accounts::policy::load_policy(&state).await.registration_enabled,
    })
}

pub async fn run_setup(
    State(state): State<AppState>,
    client: crate::accounts::client_info::ClientInfo,
    jar: CookieJar,
    Json(payload): Json<SetupRequest>,
) -> Result<(CookieJar, Json<User>), (StatusCode, String)> {
    let count: (i64,) = sqlx::query_as(sql("SELECT COUNT(*) FROM users"))
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    if count.0 > 0 {
        return Err((StatusCode::FORBIDDEN, "Setup has already been completed".to_string()));
    }

    if payload.username.is_empty() || payload.password.is_empty() || payload.email.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username, email, and password cannot be empty".to_string()));
    }

    let password_hash = crate::accounts::passwords::hash_password(&payload.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let user_id = Uuid::new_v4().to_string();

    sqlx::query(sql("INSERT INTO users (id, username, email, password_hash, is_admin, email_verified) VALUES ($1, $2, $3, $4, $5, $6)"))
    .bind(&user_id)
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(1i64)
    .bind(1i64)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user = sqlx::query_as::<_, User>(sql("SELECT id, username, email, password_hash, is_admin, email_verified FROM users WHERE id = $1"))
    .bind(&user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let cookie = crate::accounts::sessions::start_session(&state, &user.id, &client).await?;

    Ok((jar.add(cookie), Json(user)))
}

use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::{Cookie, SameSite, SignedCookieJar};
use uuid::Uuid;


use crate::{
    models::{SetupRequest, SetupStatus, User},
    AppState,
};

pub async fn setup_status(
    State(state): State<AppState>,
) -> Json<SetupStatus> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    Json(SetupStatus {
        needs_setup: count.0 == 0,
        registration_enabled: crate::account_policy::load_policy(&state).await.registration_enabled,
    })
}

pub async fn run_setup(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<SetupRequest>,
) -> Result<(SignedCookieJar, Json<User>), (StatusCode, String)> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    if count.0 > 0 {
        return Err((StatusCode::FORBIDDEN, "Setup has already been completed".to_string()));
    }

    if payload.username.is_empty() || payload.password.is_empty() || payload.email.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username, email, and password cannot be empty".to_string()));
    }

    let password_hash = crate::passwords::hash_password(&payload.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let user_id = Uuid::new_v4().to_string();

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, username, email, password_hash, is_admin) VALUES ($1, $2, $3, $4, $5) RETURNING id, username, email, password_hash, is_admin"
    )
    .bind(&user_id)
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(1i64)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut cookie = Cookie::new("session_user_id", user.id.clone());
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");

    Ok((jar.add(cookie), Json(user)))
}

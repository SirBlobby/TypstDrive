use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use uuid::Uuid;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

use crate::{
    models::{AdminCreateUserRequest, AdminUserView, UpdateUserRequest},
    AppState,
};

async fn require_admin(state: &AppState, jar: &SignedCookieJar) -> Result<String, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let is_admin: Option<(i64,)> = sqlx::query_as("SELECT is_admin FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match is_admin {
        Some((v,)) if v != 0 => Ok(user_id),
        _ => Err((StatusCode::FORBIDDEN, "Admin access required".to_string())),
    }
}

pub async fn create_user(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<AdminCreateUserRequest>,
) -> Result<(StatusCode, Json<AdminUserView>), (StatusCode, String)> {
    require_admin(&state, &jar).await?;

    if payload.username.is_empty() || payload.email.is_empty() || payload.password.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username, email, and password are required".to_string()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .to_string();

    let user_id = Uuid::new_v4().to_string();
    let is_admin = payload.is_admin.unwrap_or(false);

    let result = sqlx::query_as::<_, AdminUserView>(
        "INSERT INTO users (id, username, email, password_hash, is_admin) VALUES (?, ?, ?, ?, ?) RETURNING id, username, email, is_admin, created_at"
    )
    .bind(&user_id)
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(if is_admin { 1i64 } else { 0i64 })
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(user) => Ok((StatusCode::CREATED, Json(user))),
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err((StatusCode::CONFLICT, "Username or email already exists".to_string()))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn list_users(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<AdminUserView>>, (StatusCode, String)> {
    require_admin(&state, &jar).await?;

    let users = sqlx::query_as::<_, AdminUserView>(
        "SELECT id, username, email, is_admin, created_at FROM users ORDER BY created_at ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(users))
}

pub async fn update_user(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<AdminUserView>, (StatusCode, String)> {
    let requester_id = require_admin(&state, &jar).await?;

    if let Some(is_admin) = payload.is_admin {
        if !is_admin && requester_id == user_id {
            return Err((StatusCode::BAD_REQUEST, "Cannot remove your own admin privileges".to_string()));
        }
        sqlx::query("UPDATE users SET is_admin = ? WHERE id = ?")
            .bind(if is_admin { 1i64 } else { 0i64 })
            .bind(&user_id)
            .execute(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    let user = sqlx::query_as::<_, AdminUserView>(
        "SELECT id, username, email, is_admin, created_at FROM users WHERE id = ?"
    )
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    Ok(Json(user))
}

pub async fn delete_user(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(user_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let requester_id = require_admin(&state, &jar).await?;

    if requester_id == user_id {
        return Err((StatusCode::BAD_REQUEST, "Cannot delete your own account via admin panel".to_string()));
    }

    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

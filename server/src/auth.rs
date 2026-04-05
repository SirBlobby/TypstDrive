use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite, SignedCookieJar};
use uuid::Uuid;

use crate::{
    models::{User, RegisterRequest, LoginRequest, ChangePasswordRequest, UpdateProfileRequest, StorageStats},
    AppState,
};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<User>, (StatusCode, String)> {
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username and password cannot be empty".to_string()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .to_string();

    let user_id = Uuid::new_v4().to_string();

    let result = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, username, password_hash) VALUES (?, ?, ?) RETURNING id, username, password_hash"
    )
    .bind(&user_id)
    .bind(&payload.username)
    .bind(&password_hash)
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(user) => Ok(Json(user)),
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err((StatusCode::CONFLICT, "Username already exists".to_string()))
        }
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}

pub async fn login(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(SignedCookieJar, Json<User>), (StatusCode, String)> {
    let user = sqlx::query_as::<_, User>("SELECT id, username, password_hash FROM users WHERE username = ?")
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user = match user {
        Some(u) => u,
        None => return Err((StatusCode::UNAUTHORIZED, "Invalid username or password".to_string())),
    };

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if Argon2::default().verify_password(payload.password.as_bytes(), &parsed_hash).is_err() {
        return Err((StatusCode::UNAUTHORIZED, "Invalid username or password".to_string()));
    }

    let mut cookie = Cookie::new("session_user_id", user.id.clone());
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    
    let jar = jar.add(cookie);

    Ok((jar, Json(user)))
}

pub async fn update_profile(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<User>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    if payload.username.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username cannot be empty".to_string()));
    }

    let result = sqlx::query("UPDATE users SET username = ? WHERE id = ?")
        .bind(&payload.username)
        .bind(&user_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => {
            let user = sqlx::query_as::<_, User>("SELECT id, username, password_hash FROM users WHERE id = ?")
                .bind(&user_id)
                .fetch_optional(&state.db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;
            Ok(Json(user))
        }
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err((StatusCode::CONFLICT, "Username already exists".to_string()))
        }
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}

pub async fn logout(jar: SignedCookieJar) -> Result<(SignedCookieJar, StatusCode), (StatusCode, String)> {
    let jar = jar.remove(Cookie::from("session_user_id"));
    Ok((jar, StatusCode::OK))
}

pub async fn me(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<User>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string());

    let user_id = match user_id {
        Some(id) => id,
        None => return Err((StatusCode::UNAUTHORIZED, "Not logged in".to_string())),
    };

    let user = sqlx::query_as::<_, User>("SELECT id, username, password_hash FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match user {
        Some(u) => Ok(Json(u)),
        None => Err((StatusCode::UNAUTHORIZED, "User not found".to_string())),
    }
}

pub async fn change_password(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    if payload.current_password.is_empty() || payload.new_password.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Passwords cannot be empty".to_string()));
    }

    let user = sqlx::query_as::<_, User>("SELECT id, username, password_hash FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if Argon2::default().verify_password(payload.current_password.as_bytes(), &parsed_hash).is_err() {
        return Err((StatusCode::UNAUTHORIZED, "Invalid current password".to_string()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let new_password_hash = Argon2::default()
        .hash_password(payload.new_password.as_bytes(), &salt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .to_string();

    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&new_password_hash)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

pub async fn storage_stats(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<StorageStats>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let docs_size: (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(LENGTH(content)), 0) FROM documents WHERE owner_id = ?"
    )
    .bind(&user_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or((0,));

    let files_size: (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(LENGTH(data)), 0) FROM files WHERE owner_id = ?"
    )
    .bind(&user_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or((0,));

    let stats = StorageStats {
        documents_size_bytes: docs_size.0,
        files_size_bytes: files_size.0,
        total_size_bytes: docs_size.0 + files_size.0,
    };

    Ok(Json(stats))
}

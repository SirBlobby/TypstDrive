use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite, SignedCookieJar};
use uuid::Uuid;

use crate::{
    accounts::avatars,
    models::{User, UserProfile, RegisterRequest, LoginRequest, ChangePasswordRequest, UpdateProfileRequest},
    AppState,
};


async fn profile_for(state: &AppState, user: User) -> UserProfile {
    let avatar_updated_at = avatars::avatar_updated_at(state, &user.id).await;
    UserProfile { user, avatar_updated_at }
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<User>, (StatusCode, String)> {
    let policy = crate::accounts::policy::load_policy(&state).await;
    if !policy.registration_enabled {
        return Err((StatusCode::FORBIDDEN, "Registration is disabled on this instance".to_string()));
    }

    if payload.username.is_empty() || payload.password.is_empty() || payload.email.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username, email, and password cannot be empty".to_string()));
    }

    policy.check_email_domain(&payload.email)?;

    let password_hash = crate::accounts::passwords::hash_password(&payload.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let user_id = Uuid::new_v4().to_string();

    let result = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4) RETURNING id, username, email, password_hash, is_admin"
    )
    .bind(&user_id)
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(user) => Ok(Json(user)),
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err((StatusCode::CONFLICT, "Username or email already exists".to_string()))
        }
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}

pub async fn login(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(SignedCookieJar, Json<UserProfile>), (StatusCode, String)> {
    let user = sqlx::query_as::<_, User>("SELECT id, username, email, password_hash, is_admin FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user = match user {
        Some(u) => u,
        None => return Err((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string())),
    };

    if !crate::accounts::passwords::verify_password(&payload.password, &user.password_hash) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()));
    }

    let mut cookie = Cookie::new("session_user_id", user.id.clone());
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");

    let jar = jar.add(cookie);
    let profile = profile_for(&state, user).await;

    Ok((jar, Json(profile)))
}

pub async fn update_profile(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    if payload.username.is_empty() || payload.email.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username and email cannot be empty".to_string()));
    }

    let current = sqlx::query_as::<_, User>("SELECT id, username, email, password_hash, is_admin FROM users WHERE id = $1")
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    if current.is_admin == 0 {
        let policy = crate::accounts::policy::load_policy(&state).await;
        if payload.username != current.username && !policy.allow_username_change {
            return Err((StatusCode::FORBIDDEN, "Changing your username is disabled on this instance".to_string()));
        }
        if payload.email != current.email {
            if !policy.allow_email_change {
                return Err((StatusCode::FORBIDDEN, "Changing your email is disabled on this instance".to_string()));
            }
            policy.check_email_domain(&payload.email)?;
        }
    }

    let result = sqlx::query("UPDATE users SET username = $1, email = $2 WHERE id = $3")
        .bind(&payload.username)
        .bind(&payload.email)
        .bind(&user_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => {
            let user = sqlx::query_as::<_, User>("SELECT id, username, email, password_hash, is_admin FROM users WHERE id = $1")
                .bind(&user_id)
                .fetch_optional(&state.db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;
            Ok(Json(profile_for(&state, user).await))
        }
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err((StatusCode::CONFLICT, "Username or email already exists".to_string()))
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
) -> Result<Json<UserProfile>, (StatusCode, String)> {
    let user_id = match jar.get("session_user_id").map(|c| c.value().to_string()) {
        Some(id) => id,
        None => return Err((StatusCode::UNAUTHORIZED, "Not logged in".to_string())),
    };

    let user = sqlx::query_as::<_, User>("SELECT id, username, email, password_hash, is_admin FROM users WHERE id = $1")
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match user {
        Some(u) => Ok(Json(profile_for(&state, u).await)),
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

    let user = sqlx::query_as::<_, User>("SELECT id, username, email, password_hash, is_admin FROM users WHERE id = $1")
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    if !crate::accounts::passwords::verify_password(&payload.current_password, &user.password_hash) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid current password".to_string()));
    }

    let new_password_hash = crate::accounts::passwords::hash_password(&payload.new_password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(&new_password_hash)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

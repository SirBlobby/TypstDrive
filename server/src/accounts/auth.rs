use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use axum_extra::extract::cookie::{CookieJar, SignedCookieJar};
use uuid::Uuid;

use crate::{
    accounts::{auth_settings, avatars, client_info::ClientInfo, lookup, passwords, sessions, verification},
    models::{User, UserProfile, RegisterRequest, LoginRequest, ChangePasswordRequest, UpdateProfileRequest},
    AppState,
};
use crate::db::sql;

type ApiError = (StatusCode, String);

const LOGIN_SCOPE: &str = "login";

pub async fn profile_for(state: &AppState, user: User) -> UserProfile {
    let avatar_updated_at = avatars::avatar_updated_at(state, &user.id).await;
    let has_password = passwords::has_password(&user.password_hash);
    UserProfile { user, avatar_updated_at, has_password }
}

fn session_user_id(jar: &SignedCookieJar) -> Result<String, ApiError> {
    jar.get("session_user_id")
        .map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))
}

async fn current_user(state: &AppState, user_id: &str) -> Result<User, ApiError> {
    lookup::user_by_id(state, user_id)
        .await?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<User>, ApiError> {
    let policy = crate::accounts::policy::load_policy(&state).await;
    let settings = auth_settings::load(&state).await;
    if !policy.registration_enabled || !settings.password_login_enabled {
        return Err((StatusCode::FORBIDDEN, "Registration is disabled on this instance".to_string()));
    }

    if payload.username.is_empty() || payload.password.is_empty() || payload.email.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username, email, and password cannot be empty".to_string()));
    }

    policy.check_email_domain(&payload.email)?;

    let password_hash = passwords::hash_password(&payload.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let user_id = Uuid::new_v4().to_string();

    let inserted = sqlx::query(sql("INSERT INTO users (id, username, email, password_hash, email_verified) VALUES ($1, $2, $3, $4, $5)"))
        .bind(&user_id)
        .bind(&payload.username)
        .bind(payload.email.trim())
        .bind(&password_hash)
        .bind(0i64)
        .execute(&state.db)
        .await;

    match inserted {
        Ok(_) => {}
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            return Err((StatusCode::CONFLICT, "Username or email already exists".to_string()));
        }
        Err(err) => return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }

    let user = current_user(&state, &user_id).await?;
    verification::send_verification_quietly(&state, &user.id, &user.username, &user.email).await;
    Ok(Json(user))
}

pub async fn login(
    State(state): State<AppState>,
    client: ClientInfo,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<UserProfile>), ApiError> {
    let settings = auth_settings::load(&state).await;
    state.login_guard.ensure_allowed(LOGIN_SCOPE, &payload.email, &client.ip_address)?;

    let user = lookup::user_by_email(&state, &payload.email).await?;
    let user = match user {
        Some(user) if passwords::verify_password(&payload.password, &user.password_hash) => user,
        _ => {
            state.login_guard.record_failure(LOGIN_SCOPE, &payload.email, &client.ip_address, settings.lockout_rules());
            return Err((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()));
        }
    };

    if !settings.password_login_enabled && user.is_admin == 0 {
        return Err((StatusCode::FORBIDDEN, "Password sign-in is disabled. Use single sign-on instead.".to_string()));
    }
    if auth_settings::verification_required(&state, &settings) && user.email_verified == 0 {
        return Err((
            StatusCode::PRECONDITION_REQUIRED,
            "Verify your email address before signing in. Check your inbox for the link.".to_string(),
        ));
    }

    state.login_guard.record_success(LOGIN_SCOPE, &payload.email);
    let cookie = sessions::start_session(&state, &user.id, &client).await?;
    let profile = profile_for(&state, user).await;

    Ok((jar.add(cookie), Json(profile)))
}

pub async fn update_profile(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, ApiError> {
    let user_id = session_user_id(&jar)?;

    if payload.username.is_empty() || payload.email.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username and email cannot be empty".to_string()));
    }

    let current = current_user(&state, &user_id).await?;
    let email_changed = !payload.email.trim().eq_ignore_ascii_case(&current.email);

    if current.is_admin == 0 {
        let policy = crate::accounts::policy::load_policy(&state).await;
        if payload.username != current.username && !policy.allow_username_change {
            return Err((StatusCode::FORBIDDEN, "Changing your username is disabled on this instance".to_string()));
        }
        if email_changed {
            if !policy.allow_email_change {
                return Err((StatusCode::FORBIDDEN, "Changing your email is disabled on this instance".to_string()));
            }
            policy.check_email_domain(&payload.email)?;
        }
    }

    let email_verified = if email_changed { 0i64 } else { current.email_verified };
    let result = sqlx::query(sql("UPDATE users SET username = $1, email = $2, email_verified = $3 WHERE id = $4"))
        .bind(&payload.username)
        .bind(payload.email.trim())
        .bind(email_verified)
        .bind(&user_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => {
            let user = current_user(&state, &user_id).await?;
            if email_changed {
                verification::send_verification_quietly(&state, &user.id, &user.username, &user.email).await;
            }
            Ok(Json(profile_for(&state, user).await))
        }
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err((StatusCode::CONFLICT, "Username or email already exists".to_string()))
        }
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), ApiError> {
    sessions::end_session(&state, &headers).await;
    Ok((jar.remove(sessions::cleared_cookie()), StatusCode::OK))
}

pub async fn me(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<UserProfile>, ApiError> {
    let user_id = session_user_id(&jar)?;
    let user = current_user(&state, &user_id).await?;
    Ok(Json(profile_for(&state, user).await))
}

pub async fn change_password(
    State(state): State<AppState>,
    sessions::CurrentSession(session): sessions::CurrentSession,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<StatusCode, ApiError> {
    if payload.new_password.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Password cannot be empty".to_string()));
    }

    let user = current_user(&state, &session.user_id).await?;
    if passwords::has_password(&user.password_hash)
        && !passwords::verify_password(&payload.current_password, &user.password_hash)
    {
        return Err((StatusCode::UNAUTHORIZED, "Invalid current password".to_string()));
    }

    let new_password_hash = passwords::hash_password(&payload.new_password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    sqlx::query(sql("UPDATE users SET password_hash = $1 WHERE id = $2"))
        .bind(&new_password_hash)
        .bind(&session.user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sessions::end_all_sessions(&state, &session.user_id, Some(&session.session_id)).await?;
    Ok(StatusCode::OK)
}

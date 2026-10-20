use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use uuid::Uuid;

use crate::{
    accounts::client_info::ClientInfo,
    admin::audit,
    models::{AdminCreateUserRequest, AdminUserView, UpdateUserRequest},
    AppState,
};
use crate::db::sql;

pub(crate) async fn require_admin(state: &AppState, jar: &SignedCookieJar) -> Result<String, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let is_admin: Option<(i64,)> = sqlx::query_as(sql("SELECT is_admin FROM users WHERE id = $1"))
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match is_admin {
        Some((v,)) if v != 0 => Ok(user_id),
        _ => Err((StatusCode::FORBIDDEN, "Admin access required".to_string())),
    }
}

async fn find_user(state: &AppState, user_id: &str) -> Result<Option<AdminUserView>, sqlx::Error> {
    sqlx::query_as::<_, AdminUserView>(sql(
        "SELECT id, username, email, is_admin, email_verified, created_at FROM users WHERE id = $1 AND account_kind = 'user'",
    ))
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

pub async fn create_user(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
    Json(payload): Json<AdminCreateUserRequest>,
) -> Result<(StatusCode, Json<AdminUserView>), (StatusCode, String)> {
    let admin_id = require_admin(&state, &jar).await?;

    if payload.username.is_empty() || payload.email.is_empty() || payload.password.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username, email, and password are required".to_string()));
    }

    let password_hash = crate::accounts::passwords::hash_password(&payload.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let user_id = Uuid::new_v4().to_string();
    let is_admin = payload.is_admin.unwrap_or(false);

    let inserted = sqlx::query(sql("INSERT INTO users (id, username, email, password_hash, is_admin, email_verified) VALUES ($1, $2, $3, $4, $5, $6)"))
        .bind(&user_id)
        .bind(&payload.username)
        .bind(payload.email.trim())
        .bind(&password_hash)
        .bind(if is_admin { 1i64 } else { 0i64 })
        .bind(1i64)
        .execute(&state.db)
        .await;

    let result = match inserted {
        Ok(_) => find_user(&state, &user_id).await.and_then(|user| user.ok_or(sqlx::Error::RowNotFound)),
        Err(err) => Err(err),
    };

    match result {
        Ok(user) => {
            let details = format!("email {}, admin {}", user.email, yes_no(is_admin));
            audit::record(&state, &admin_id, &client, "user.create", &user.username, &details).await;
            Ok((StatusCode::CREATED, Json(user)))
        }
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
        sql("SELECT id, username, email, is_admin, email_verified, created_at FROM users WHERE account_kind = 'user' ORDER BY created_at ASC")
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(users))
}

pub async fn update_user(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<AdminUserView>, (StatusCode, String)> {
    let requester_id = require_admin(&state, &jar).await?;
    let internal = |e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string());

    let existing = find_user(&state, &user_id)
        .await
        .map_err(internal)?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    if let Some(is_admin) = payload.is_admin {
        if !is_admin && requester_id == user_id {
            return Err((StatusCode::BAD_REQUEST, "Cannot remove your own admin privileges".to_string()));
        }
        sqlx::query(sql("UPDATE users SET is_admin = $1 WHERE id = $2"))
            .bind(if is_admin { 1i64 } else { 0i64 })
            .bind(&user_id)
            .execute(&state.db)
            .await
            .map_err(internal)?;
        if is_admin != (existing.is_admin != 0) {
            let action = if is_admin { "user.grant_admin" } else { "user.revoke_admin" };
            audit::record(&state, &requester_id, &client, action, &existing.username, "").await;
        }
    }

    if let Some(email_verified) = payload.email_verified {
        sqlx::query(sql("UPDATE users SET email_verified = $1 WHERE id = $2"))
            .bind(if email_verified { 1i64 } else { 0i64 })
            .bind(&user_id)
            .execute(&state.db)
            .await
            .map_err(internal)?;
        if email_verified != (existing.email_verified != 0) {
            let details = format!("verified {}", yes_no(email_verified));
            audit::record(&state, &requester_id, &client, "user.email_verification", &existing.username, &details).await;
        }
    }

    let user = find_user(&state, &user_id)
        .await
        .map_err(internal)?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    Ok(Json(user))
}

pub async fn delete_user(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
    Path(user_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let requester_id = require_admin(&state, &jar).await?;

    if requester_id == user_id {
        return Err((StatusCode::BAD_REQUEST, "Cannot delete your own account via admin panel".to_string()));
    }

    let existing = find_user(&state, &user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    crate::teams::handle_departing_user(&state, &user_id).await;

    sqlx::query(sql("DELETE FROM users WHERE id = $1"))
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::sharing::access::revoke_user_sessions(&state, &user_id).await;
    audit::record(&state, &requester_id, &client, "user.delete", &existing.username, &existing.email).await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn revoke_user_sessions(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let requester_id = require_admin(&state, &jar).await?;
    let existing = find_user(&state, &user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let revoked = crate::accounts::sessions::end_all_sessions(&state, &user_id, None).await?;
    crate::sharing::access::revoke_user_sessions(&state, &user_id).await;
    audit::record(&state, &requester_id, &client, "user.sign_out_everywhere", &existing.username, "").await;

    Ok(Json(serde_json::json!({ "revoked": revoked })))
}

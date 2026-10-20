use super::*;
use crate::db::sql;

const DESKTOP_LOGIN_SCOPE: &str = "desktop-login";

#[derive(Deserialize)]
pub struct DeviceLoginRequest {
    pub email: String,
    pub password: String,
    pub device_name: Option<String>,
    pub client_version: Option<String>,
}

#[derive(Serialize)]
pub struct DeviceLoginResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub email: String,
}

pub(super) fn ensure_supported_client(client_version: Option<&str>) -> Result<(), (StatusCode, String)> {
    match client_version {
        Some(version) if !version_at_least(version, MIN_DESKTOP_VERSION) => Err((
            StatusCode::UPGRADE_REQUIRED,
            format!(
                "This server requires typst-desktop v{} or newer (you have v{}). Please update the app.",
                MIN_DESKTOP_VERSION, version
            ),
        )),
        _ => Ok(()),
    }
}

pub(super) async fn issue_device_token(
    state: &AppState,
    user_id: &str,
    device_name: Option<String>,
) -> Result<String, (StatusCode, String)> {
    let token = generate_token();
    let device_name = device_name
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| "Typst Desktop".to_string());

    sqlx::query(
        sql("INSERT INTO device_tokens (id, user_id, name, token_hash, created_at) VALUES ($1, $2, $3, $4, $5)"),
    )
    .bind(Uuid::new_v4().to_string())
    .bind(user_id)
    .bind(&device_name)
    .bind(hash_token(&token))
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(token)
}

pub async fn login(
    State(state): State<AppState>,
    client: crate::accounts::client_info::ClientInfo,
    Json(payload): Json<DeviceLoginRequest>,
) -> Result<Json<DeviceLoginResponse>, (StatusCode, String)> {
    ensure_supported_client(payload.client_version.as_deref())?;

    let settings = crate::accounts::auth_settings::load(&state).await;
    state.login_guard.ensure_allowed(DESKTOP_LOGIN_SCOPE, &payload.email, &client.ip_address)?;

    let user = crate::accounts::lookup::user_by_email(&state, &payload.email).await?;
    let stored_hash = user.as_ref().map(|user| user.password_hash.as_str());
    let verified = crate::accounts::passwords::verify_login(&payload.password, stored_hash).await;
    let user = match user {
        Some(user) if verified => user,
        _ => {
            state.login_guard.record_failure(DESKTOP_LOGIN_SCOPE, &payload.email, &client.ip_address, settings.lockout_rules());
            return Err((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()));
        }
    };

    if !settings.password_login_enabled && user.is_admin == 0 {
        return Err((StatusCode::FORBIDDEN, "Password sign-in is disabled on this server".to_string()));
    }
    if crate::accounts::auth_settings::verification_required(&state, &settings) && user.email_verified == 0 {
        return Err((StatusCode::PRECONDITION_REQUIRED, "Verify your email address before signing in".to_string()));
    }
    state.login_guard.record_success(DESKTOP_LOGIN_SCOPE, &payload.email);

    let token = issue_device_token(&state, &user.id, payload.device_name).await?;

    Ok(Json(DeviceLoginResponse {
        token,
        user_id: user.id,
        username: user.username,
        email: user.email,
    }))
}

#[derive(Serialize)]
pub struct DeviceUser {
    pub user_id: String,
    pub username: String,
    pub email: String,
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DeviceUser>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let user = sqlx::query_as::<_, User>(
        sql("SELECT id, username, email, password_hash, is_admin, email_verified FROM users WHERE id = $1"),
    )
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    Ok(Json(DeviceUser {
        user_id: user.id,
        username: user.username,
        email: user.email,
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, String)> {
    let token = bearer_token(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        "Missing Authorization header".to_string(),
    ))?;

    let token_id = sqlx::query_as::<_, (String,)>(sql("SELECT id FROM device_tokens WHERE token_hash = $1"))
        .bind(hash_token(&token))
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query(sql("DELETE FROM device_tokens WHERE token_hash = $1"))
        .bind(hash_token(&token))
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some((token_id,)) = token_id {
        crate::sharing::access::revoke_device_sessions(&state, &token_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
}

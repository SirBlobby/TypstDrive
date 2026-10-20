use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Serialize;
use sqlx::FromRow;

use crate::accounts::{lookup, passwords};
use crate::db::sql;
use crate::AppState;

type ApiError = (StatusCode, String);

#[derive(Serialize, FromRow)]
pub struct IdentityView {
    pub id: String,
    pub provider: String,
    pub email: Option<String>,
    pub created_at: String,
    #[sqlx(default)]
    pub provider_name: String,
}

fn session_user_id(jar: &SignedCookieJar) -> Result<String, ApiError> {
    jar.get("session_user_id")
        .map(|cookie| cookie.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))
}

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

async fn identities_of(state: &AppState, user_id: &str) -> Result<Vec<IdentityView>, ApiError> {
    let mut identities = sqlx::query_as::<_, IdentityView>(sql(
        "SELECT id, provider, email, created_at FROM user_identities WHERE user_id = $1 ORDER BY created_at ASC",
    ))
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    for identity in &mut identities {
        identity.provider_name = state
            .sso
            .configured_providers()
            .find(|provider| provider.id == identity.provider)
            .map(|provider| provider.name.clone())
            .unwrap_or_else(|| identity.provider.clone());
    }
    Ok(identities)
}

pub async fn list_identities(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<IdentityView>>, ApiError> {
    let user_id = session_user_id(&jar)?;
    Ok(Json(identities_of(&state, &user_id).await?))
}

pub async fn unlink_identity(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(identity_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let user_id = session_user_id(&jar)?;
    let user = lookup::user_by_id(&state, &user_id)
        .await?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    let identities = identities_of(&state, &user_id).await?;
    if !identities.iter().any(|identity| identity.id == identity_id) {
        return Err((StatusCode::NOT_FOUND, "Connected account not found".to_string()));
    }
    if identities.len() == 1 && !passwords::has_password(&user.password_hash) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Set a password before disconnecting your only sign-in method".to_string(),
        ));
    }

    sqlx::query(sql("DELETE FROM user_identities WHERE id = $1 AND user_id = $2"))
        .bind(&identity_id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

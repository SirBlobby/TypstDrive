use axum::extract::{Multipart, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;

use super::{current_branding, ApiError, Branding};
use crate::accounts::avatars::compress_image;
use crate::accounts::client_info::ClientInfo;
use crate::admin::users::require_admin;
use crate::db::sql;
use crate::AppState;

const LOGO_ASSET: &str = "logo";
const MAX_LOGO_DIMENSION: u32 = 256;
const LOGO_MIME_TYPE: &str = "image/webp";

fn server_error(error: impl ToString) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

pub async fn logo_updated_at(state: &AppState) -> Option<String> {
    sqlx::query_as::<_, (String,)>(sql("SELECT updated_at FROM site_assets WHERE asset_name = $1"))
        .bind(LOGO_ASSET)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .map(|(updated_at,)| updated_at)
}

async fn delete_logo(state: &AppState) -> Result<(), ApiError> {
    sqlx::query(sql("DELETE FROM site_assets WHERE asset_name = $1"))
        .bind(LOGO_ASSET)
        .execute(&state.db)
        .await
        .map_err(server_error)?;
    Ok(())
}

pub async fn get_logo(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let logo = sqlx::query_as::<_, (Vec<u8>,)>(sql("SELECT data FROM site_assets WHERE asset_name = $1"))
        .bind(LOGO_ASSET)
        .fetch_optional(&state.db)
        .await
        .map_err(server_error)?;

    match logo {
        Some((data,)) => Ok((
            [
                (header::CONTENT_TYPE, LOGO_MIME_TYPE),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
            ],
            data,
        )),
        None => Err((StatusCode::NOT_FOUND, "No logo uploaded".to_string())),
    }
}

pub async fn upload_logo(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
    mut multipart: Multipart,
) -> Result<Json<Branding>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;

    let field = multipart
        .next_field()
        .await
        .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))?
        .ok_or((StatusCode::BAD_REQUEST, "No image provided".to_string()))?;
    let source = field
        .bytes()
        .await
        .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))?
        .to_vec();

    let compressed = tokio::task::spawn_blocking(move || compress_image(&source, MAX_LOGO_DIMENSION))
        .await
        .map_err(server_error)?
        .map_err(|error| (StatusCode::BAD_REQUEST, error))?;

    let updated_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    delete_logo(&state).await?;
    sqlx::query(sql("INSERT INTO site_assets (asset_name, data, updated_at) VALUES ($1, $2, $3)"))
        .bind(LOGO_ASSET)
        .bind(&compressed)
        .bind(&updated_at)
        .execute(&state.db)
        .await
        .map_err(server_error)?;

    crate::admin::audit::record(&state, &admin_id, &client, "settings.branding", "", "logo uploaded").await;
    Ok(Json(current_branding(&state).await))
}

pub async fn remove_logo(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
) -> Result<Json<Branding>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    delete_logo(&state).await?;
    crate::admin::audit::record(&state, &admin_id, &client, "settings.branding", "", "logo removed").await;
    Ok(Json(current_branding(&state).await))
}

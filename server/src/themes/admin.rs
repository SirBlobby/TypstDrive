use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;

use super::palette::{Theme, ThemeInput};
use super::{cache, check_name_free, ensure_exists, internal_error, store, ApiError};
use crate::accounts::client_info::ClientInfo;
use crate::admin::audit;
use crate::admin::users::require_admin;
use crate::AppState;

const MAX_SITE_THEMES: usize = 50;

pub async fn create_site_theme(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    client: ClientInfo,
    Json(input): Json<ThemeInput>,
) -> Result<Json<Theme>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    let input = input.normalized()?;
    {
        let cache = state.themes.read().await;
        if cache.site.len() >= MAX_SITE_THEMES {
            return Err((StatusCode::BAD_REQUEST, format!("A server can have up to {MAX_SITE_THEMES} site themes")));
        }
        check_name_free(&cache.site, &input.name, None)?;
    }

    let id = store::insert(&state.db, None, &input).await.map_err(internal_error)?;
    cache::changed(&state, None).await;
    audit::record(&state, &admin_id, &client, "settings.themes", &id, &format!("added theme \"{}\"", input.name)).await;
    Ok(Json(input.into_theme(id)))
}

pub async fn update_site_theme(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    client: ClientInfo,
    Json(input): Json<ThemeInput>,
) -> Result<Json<Theme>, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    let input = input.normalized()?;
    {
        let cache = state.themes.read().await;
        ensure_exists(&cache.site, &id)?;
        check_name_free(&cache.site, &input.name, Some(&id))?;
    }

    store::update(&state.db, &id, None, &input).await.map_err(internal_error)?;
    cache::changed(&state, None).await;
    audit::record(&state, &admin_id, &client, "settings.themes", &id, &format!("changed theme \"{}\"", input.name)).await;
    Ok(Json(input.into_theme(id)))
}

pub async fn delete_site_theme(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    client: ClientInfo,
) -> Result<StatusCode, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    let name = {
        let cache = state.themes.read().await;
        ensure_exists(&cache.site, &id)?;
        cache.site.iter().find(|theme| theme.id == id).map(|theme| theme.name.clone()).unwrap_or_default()
    };

    store::delete(&state.db, &id, None).await.map_err(internal_error)?;
    cache::changed(&state, None).await;
    audit::record(&state, &admin_id, &client, "settings.themes", &id, &format!("removed theme \"{name}\"")).await;
    Ok(StatusCode::NO_CONTENT)
}

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Serialize;

use super::palette::{Theme, ThemeInput};
use super::{cache, check_name_free, ensure_exists, internal_error, store, ApiError};
use crate::accounts::signed_in::SignedInUser;
use crate::AppState;

const MAX_PERSONAL_THEMES: usize = 20;

#[derive(Serialize)]
pub struct ThemeList {
    pub site: Vec<Theme>,
    pub personal: Vec<Theme>,
    pub personal_enabled: bool,
}

fn personal_themes_off() -> ApiError {
    (StatusCode::FORBIDDEN, "Personal themes are turned off on this server".to_string())
}

pub async fn list_themes(State(state): State<AppState>, jar: SignedCookieJar) -> Json<ThemeList> {
    let user_id = jar.get("session_user_id").map(|cookie| cookie.value().to_string());
    let cache = state.themes.read().await;
    let personal = match user_id {
        Some(user_id) if cache.personal_enabled => cache.personal_for(&user_id).to_vec(),
        _ => Vec::new(),
    };
    Json(ThemeList { site: cache.site.clone(), personal, personal_enabled: cache.personal_enabled })
}

pub async fn create_theme(
    State(state): State<AppState>,
    SignedInUser(user_id): SignedInUser,
    Json(input): Json<ThemeInput>,
) -> Result<Json<Theme>, ApiError> {
    let input = input.normalized()?;
    {
        let cache = state.themes.read().await;
        if !cache.personal_enabled {
            return Err(personal_themes_off());
        }
        let mine = cache.personal_for(&user_id);
        if mine.len() >= MAX_PERSONAL_THEMES {
            return Err((StatusCode::BAD_REQUEST, format!("You can have up to {MAX_PERSONAL_THEMES} themes")));
        }
        check_name_free(mine, &input.name, None)?;
    }

    let id = store::insert(&state.db, Some(&user_id), &input).await.map_err(internal_error)?;
    cache::changed(&state, Some(&user_id)).await;
    Ok(Json(input.into_theme(id)))
}

pub async fn update_theme(
    State(state): State<AppState>,
    Path(id): Path<String>,
    SignedInUser(user_id): SignedInUser,
    Json(input): Json<ThemeInput>,
) -> Result<Json<Theme>, ApiError> {
    let input = input.normalized()?;
    {
        let cache = state.themes.read().await;
        if !cache.personal_enabled {
            return Err(personal_themes_off());
        }
        let mine = cache.personal_for(&user_id);
        ensure_exists(mine, &id)?;
        check_name_free(mine, &input.name, Some(&id))?;
    }

    store::update(&state.db, &id, Some(&user_id), &input).await.map_err(internal_error)?;
    cache::changed(&state, Some(&user_id)).await;
    Ok(Json(input.into_theme(id)))
}

pub async fn delete_theme(
    State(state): State<AppState>,
    Path(id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<StatusCode, ApiError> {
    ensure_exists(state.themes.read().await.personal_for(&user_id), &id)?;
    store::delete(&state.db, &id, Some(&user_id)).await.map_err(internal_error)?;
    cache::changed(&state, Some(&user_id)).await;
    Ok(StatusCode::NO_CONTENT)
}

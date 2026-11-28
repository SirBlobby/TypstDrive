use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_extra::extract::cookie::{Cookie, SameSite, SignedCookieJar};
use serde::{Deserialize, Serialize};

use super::target::{self, record_view, LinkTarget};
use super::{cache, Format};
use crate::accounts::client_info::ClientInfo;
use crate::accounts::login_guard::LockoutRules;
use crate::compile::speaker_notes::Note;
use crate::AppState;

const UNLOCK_COOKIE_PREFIX: &str = "speaker_unlock_";
const UNLOCK_COOKIE_PATH: &str = "/api/public-links";
const UNLOCK_HOURS: i64 = 12;
const UNLOCK_SCOPE: &str = "speaker_password";

const UNLOCK_ATTEMPT_RULES: LockoutRules = LockoutRules {
    max_failures: 10,
    lockout: std::time::Duration::from_secs(5 * 60),
};

#[derive(Serialize, Deserialize, Clone)]
struct PublicSlide {
    svg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<Note>,
}

#[derive(Serialize)]
pub struct Presentation {
    title: String,
    slides: Vec<PublicSlide>,
}

#[derive(Deserialize)]
pub struct UnlockRequest {
    password: String,
}

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "Public link not found").into_response()
}

enum Unavailable {
    Missing,
    CannotCompile,
}

impl IntoResponse for Unavailable {
    fn into_response(self) -> Response {
        match self {
            Self::Missing => not_found(),
            Self::CannotCompile => {
                (StatusCode::UNPROCESSABLE_ENTITY, "This document has errors and cannot be compiled right now").into_response()
            }
        }
    }
}

fn decode(bytes: &[u8]) -> Option<Vec<PublicSlide>> {
    serde_json::from_slice(bytes).ok()
}

async fn compiled_slides(state: &AppState, link: &LinkTarget) -> Result<Vec<PublicSlide>, Unavailable> {
    let key = cache::slides_key(&link.id);
    if let Some(slides) = cache::fresh(&key).and_then(|(_, bytes)| decode(&bytes)) {
        return Ok(slides);
    }
    let lock = cache::compile_lock(&key);
    let _compiling = lock.lock().await;
    if let Some(slides) = cache::fresh(&key).and_then(|(_, bytes)| decode(&bytes)) {
        return Ok(slides);
    }

    let input = target::compile_input(state, link).await.ok_or(Unavailable::Missing)?;
    let fingerprint = cache::fingerprint(&input);
    let previous = cache::latest(&key);
    if let Some((cached_fingerprint, bytes)) = &previous {
        if *cached_fingerprint == fingerprint {
            cache::touch(&key);
            return decode(bytes).ok_or(Unavailable::CannotCompile);
        }
    }
    let fall_back = |previous: Option<(String, Vec<u8>)>| previous.and_then(|(_, bytes)| decode(&bytes)).ok_or(Unavailable::CannotCompile);
    if cache::has_failed(&key, &fingerprint) {
        return fall_back(previous);
    }

    match crate::compile::render::preview(input).await {
        Ok(preview) => {
            let notes = preview.notes.unwrap_or_default();
            let slides: Vec<PublicSlide> = preview
                .svgs
                .into_iter()
                .enumerate()
                .map(|(index, svg)| PublicSlide { svg, note: notes.get(index).cloned().flatten() })
                .collect();
            if let Ok(bytes) = serde_json::to_vec(&slides) {
                cache::remember(&key, &fingerprint, &bytes);
            }
            Ok(slides)
        }
        Err(_) => {
            cache::remember_failure(&key, &fingerprint);
            fall_back(previous)
        }
    }
}

fn unlock_cookie_name(link: &LinkTarget) -> String {
    format!("{UNLOCK_COOKIE_PREFIX}{}", link.id)
}

fn unlock_proof(link: &LinkTarget) -> String {
    let password_hash = link.speaker_password_hash.as_deref().unwrap_or_default();
    crate::hashing::sha256_hex(format!("{}:{password_hash}", link.id).as_bytes())
}

fn is_unlocked(jar: &SignedCookieJar, link: &LinkTarget) -> bool {
    jar.get(&unlock_cookie_name(link)).is_some_and(|cookie| cookie.value() == unlock_proof(link))
}

fn unlock_cookie(link: &LinkTarget) -> Cookie<'static> {
    let mut cookie = Cookie::new(unlock_cookie_name(link), unlock_proof(link));
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path(UNLOCK_COOKIE_PATH);
    cookie.set_secure(crate::site_url::uses_https());
    cookie.set_max_age(cookie::time::Duration::hours(UNLOCK_HOURS));
    cookie
}

fn private_json(presentation: Presentation) -> Response {
    ([(header::CACHE_CONTROL, "no-store")], Json(presentation)).into_response()
}

pub async fn audience_slides(State(state): State<AppState>, Path(slug): Path<String>) -> Response {
    let Some(link) = target::find(&state, &slug, Format::Slides).await else {
        return not_found();
    };
    let slides = match compiled_slides(&state, &link).await {
        Ok(slides) => slides,
        Err(unavailable) => return unavailable.into_response(),
    };
    record_view(&state, &link.id, Format::Slides).await;
    let slides = slides.into_iter().map(|slide| PublicSlide { note: None, ..slide }).collect();
    private_json(Presentation { title: link.title.unwrap_or_default(), slides })
}

pub async fn speaker_slides(State(state): State<AppState>, Path(slug): Path<String>, jar: SignedCookieJar) -> Response {
    let Some(link) = target::find(&state, &slug, Format::Speaker).await else {
        return not_found();
    };
    if !is_unlocked(&jar, &link) {
        return (StatusCode::UNAUTHORIZED, "Enter the speaker password").into_response();
    }
    let slides = match compiled_slides(&state, &link).await {
        Ok(slides) => slides,
        Err(unavailable) => return unavailable.into_response(),
    };
    record_view(&state, &link.id, Format::Speaker).await;
    private_json(Presentation { title: link.title.unwrap_or_default(), slides })
}

pub async fn unlock_speaker(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    client: ClientInfo,
    jar: SignedCookieJar,
    Json(payload): Json<UnlockRequest>,
) -> Result<(SignedCookieJar, StatusCode), (StatusCode, String)> {
    let link = target::find(&state, &slug, Format::Speaker)
        .await
        .ok_or((StatusCode::NOT_FOUND, "Public link not found".to_string()))?;
    state.login_guard.ensure_allowed(UNLOCK_SCOPE, &link.id, &client.ip_address)?;
    let password_hash = link.speaker_password_hash.clone().unwrap_or_default();
    if !crate::accounts::passwords::verify_password(&payload.password, &password_hash).await {
        state.login_guard.record_failure(UNLOCK_SCOPE, &link.id, &client.ip_address, UNLOCK_ATTEMPT_RULES);
        return Err((StatusCode::UNAUTHORIZED, "Wrong password".to_string()));
    }
    state.login_guard.record_success(UNLOCK_SCOPE, &link.id);
    Ok((jar.add(unlock_cookie(&link)), StatusCode::NO_CONTENT))
}

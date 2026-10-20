use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum_extra::extract::cookie::{Key, SignedCookieJar};

use crate::AppState;

pub struct SignedInUser(pub String);

fn not_logged_in() -> (StatusCode, String) {
    (StatusCode::UNAUTHORIZED, "Not logged in".to_string())
}

impl FromRequestParts<AppState> for SignedInUser {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        if parts.headers.contains_key(AUTHORIZATION) {
            let user_id = crate::desktop::authenticate(state, &parts.headers).await?;
            return Ok(SignedInUser(user_id));
        }
        let jar = SignedCookieJar::<Key>::from_request_parts(parts, state).await.map_err(|_| not_logged_in())?;
        jar.get("session_user_id")
            .map(|cookie| SignedInUser(cookie.value().to_string()))
            .ok_or_else(not_logged_in)
    }
}

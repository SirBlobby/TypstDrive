use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Serialize;
use typstyle_core::{Config, Typstyle};

use crate::compile::handlers::CompileRequest;
use crate::compile::pandoc::may_convert;
use crate::AppState;

const MAX_WIDTH: usize = 120;

#[derive(Serialize)]
pub struct FormatResponse {
    pub text: String,
}

pub async fn format_handler(State(state): State<AppState>, jar: SignedCookieJar, Json(payload): Json<CompileRequest>) -> Response {
    let user_id = jar.get("session_user_id").map(|cookie| cookie.value().to_string());
    if !may_convert(&state, &payload, &user_id).await {
        return (StatusCode::UNAUTHORIZED, "Sign in to format documents").into_response();
    }

    let text = payload.text.unwrap_or_default();
    let formatted = tokio::task::spawn_blocking(move || Typstyle::new(Config::new().with_width(MAX_WIDTH)).format_text(text).render()).await;
    match formatted {
        Ok(Ok(text)) => Json(FormatResponse { text }).into_response(),
        Ok(Err(error)) => (StatusCode::UNPROCESSABLE_ENTITY, error.to_string()).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Formatting failed").into_response(),
    }
}

use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response}, Json};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};

use crate::compile::compiler::{ClickTarget, PagePoint};
use crate::compile::handlers::{compile_input, CompileRequest};
use crate::compile::render;
use crate::AppState;

#[derive(Deserialize)]
pub struct CursorRequest {
    #[serde(flatten)]
    compile: CompileRequest,
    /// Project path of the file the cursor is in; defaults to the entrypoint.
    path: Option<String>,
    /// UTF-16 offset, as CodeMirror counts it.
    offset: usize,
}

#[derive(Deserialize)]
pub struct ClickRequest {
    #[serde(flatten)]
    compile: CompileRequest,
    page: usize,
    x: f64,
    y: f64,
}

#[derive(Serialize)]
pub struct CursorResponse {
    positions: Vec<PagePoint>,
}

#[derive(Serialize)]
pub struct ClickResponse {
    target: Option<ClickTarget>,
}

fn unauthorized() -> Response {
    (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
}

pub async fn cursor_handler(State(state): State<AppState>, jar: SignedCookieJar, Json(payload): Json<CursorRequest>) -> Response {
    let user_id = jar.get("session_user_id").map(|cookie| cookie.value().to_string());
    let Some(input) = compile_input(&state, &payload.compile, &user_id).await else {
        return unauthorized();
    };
    let path = payload.path.unwrap_or_else(|| input.entrypoint.clone());
    Json(CursorResponse { positions: render::cursor_positions(input, path, payload.offset).await }).into_response()
}

pub async fn click_handler(State(state): State<AppState>, jar: SignedCookieJar, Json(payload): Json<ClickRequest>) -> Response {
    let user_id = jar.get("session_user_id").map(|cookie| cookie.value().to_string());
    let Some(input) = compile_input(&state, &payload.compile, &user_id).await else {
        return unauthorized();
    };
    Json(ClickResponse { target: render::click_target(input, payload.page, payload.x, payload.y).await }).into_response()
}

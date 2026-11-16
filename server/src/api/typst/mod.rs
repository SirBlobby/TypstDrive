mod request;
mod wrap;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use axum::Json;

use crate::api::authentication::{authenticate, ApiCaller, ApiScope};
use crate::api::options::{bad_request, check_length, page_image, RenderFormat, PAGED_FORMATS};
use crate::api::render::InlineFile;
use crate::api::source_render::{check_job, render_job, Language, SourceJob};
use crate::AppState;
use request::{CheckRequest, MathRequest, SnippetRequest};
use wrap::{Layout, Wrapped, WIDTH_UNITS};

async fn caller(state: &AppState, headers: &HeaderMap) -> Result<ApiCaller, Response> {
    authenticate(state, headers, ApiScope::Render).await
}

fn wrapped_job(kind: &'static str, wrapped: Wrapped, files: Option<Vec<InlineFile>>) -> SourceJob {
    SourceJob { language: Language::Typst, kind, text: wrapped.text, lines_before_code: wrapped.lines_before_code, files }
}

pub async fn render_math(State(state): State<AppState>, headers: HeaderMap, Json(request): Json<MathRequest>) -> Response {
    let caller = match caller(&state, &headers).await {
        Ok(caller) => caller,
        Err(response) => return response,
    };
    if request.math.trim().is_empty() {
        return bad_request("math cannot be empty");
    }
    let checked = RenderFormat::choose(request.format.as_deref(), RenderFormat::Svg, PAGED_FORMATS)
        .and_then(|format| page_image(None, request.scale).map(|image| (format, image)));
    let (format, image) = match checked {
        Ok(choices) => choices,
        Err(message) => return bad_request(message),
    };
    let display = request.display.unwrap_or(true);
    let layout = Layout { preamble: request.preamble.as_deref(), width: None, transparent: request.transparent.unwrap_or(false), margin: "2pt" };
    let kind = if display { "math-display" } else { "math-inline" };
    let job = wrapped_job(kind, wrap::math_document(&request.math, display, &layout), None);
    render_job(&state, &caller, job, format, image).await
}

pub async fn render_snippet(State(state): State<AppState>, headers: HeaderMap, Json(request): Json<SnippetRequest>) -> Response {
    let caller = match caller(&state, &headers).await {
        Ok(caller) => caller,
        Err(response) => return response,
    };
    if request.code.trim().is_empty() {
        return bad_request("code cannot be empty");
    }
    let checked = RenderFormat::choose(request.format.as_deref(), RenderFormat::Svg, PAGED_FORMATS)
        .and_then(|format| request.width.as_deref().map(|width| check_length(width, &WIDTH_UNITS)).unwrap_or(Ok(())).map(|_| format))
        .and_then(|format| page_image(None, request.scale).map(|image| (format, image)));
    let (format, image) = match checked {
        Ok(choices) => choices,
        Err(message) => return bad_request(message),
    };
    let layout = Layout {
        preamble: request.preamble.as_deref(),
        width: request.width.as_deref(),
        transparent: request.transparent.unwrap_or(false),
        margin: "4pt",
    };
    let job = wrapped_job("snippet", wrap::snippet_document(&request.code, &layout), request.files);
    render_job(&state, &caller, job, format, image).await
}

pub async fn check_document(State(state): State<AppState>, headers: HeaderMap, Json(request): Json<CheckRequest>) -> Response {
    let caller = match caller(&state, &headers).await {
        Ok(caller) => caller,
        Err(response) => return response,
    };
    if request.code.trim().is_empty() {
        return bad_request("code cannot be empty");
    }
    check_job(&state, &caller, SourceJob::document(Language::Typst, request.code, request.files)).await
}

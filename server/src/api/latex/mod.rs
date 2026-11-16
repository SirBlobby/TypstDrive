mod request;
mod wrap;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::api::authentication::{authenticate, ApiCaller, ApiScope};
use crate::api::options::{bad_request, page_image, RenderFormat, PAGED_FORMATS};
use crate::api::source_render::{check_job, render_job, Language, SourceJob};
use crate::AppState;
use request::{CheckRequest, DocumentRequest, MathRequest, SnippetRequest};

async fn caller(state: &AppState, headers: &HeaderMap) -> Result<ApiCaller, Response> {
    let caller = authenticate(state, headers, ApiScope::Render).await?;
    crate::compile::latex_policy::check_enabled().map_err(IntoResponse::into_response)?;
    Ok(caller)
}

fn wrapped_job(kind: &'static str, wrapped: wrap::Wrapped, files: Option<Vec<crate::api::render::InlineFile>>) -> SourceJob {
    SourceJob { language: Language::Latex, kind, text: wrapped.text, lines_before_code: wrapped.lines_before_code, files }
}

pub async fn render_document(State(state): State<AppState>, headers: HeaderMap, Json(request): Json<DocumentRequest>) -> Response {
    let caller = match caller(&state, &headers).await {
        Ok(caller) => caller,
        Err(response) => return response,
    };
    if request.code.trim().is_empty() {
        return bad_request("code cannot be empty");
    }
    let checked = RenderFormat::choose(request.format.as_deref(), RenderFormat::Pdf, PAGED_FORMATS)
        .and_then(|format| page_image(request.page, request.scale).map(|image| (format, image)));
    let (format, image) = match checked {
        Ok(choices) => choices,
        Err(message) => return bad_request(message),
    };
    render_job(&state, &caller, SourceJob::document(Language::Latex, request.code, request.files), format, image).await
}

pub async fn render_math(State(state): State<AppState>, headers: HeaderMap, Json(request): Json<MathRequest>) -> Response {
    let caller = match caller(&state, &headers).await {
        Ok(caller) => caller,
        Err(response) => return response,
    };
    if request.math.trim().is_empty() {
        return bad_request("math cannot be empty");
    }
    let packages = request.packages.unwrap_or_default();
    let checked = RenderFormat::choose(request.format.as_deref(), RenderFormat::Svg, PAGED_FORMATS)
        .and_then(|format| wrap::check_packages(&packages).map(|_| format))
        .and_then(|format| page_image(None, request.scale).map(|image| (format, image)));
    let (format, image) = match checked {
        Ok(choices) => choices,
        Err(message) => return bad_request(message),
    };
    let display = request.display.unwrap_or(true);
    let kind = if display { "math-display" } else { "math-inline" };
    let job = wrapped_job(kind, wrap::math_document(&request.math, display, &packages), None);
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
    let packages = request.packages.unwrap_or_default();
    let checked = RenderFormat::choose(request.format.as_deref(), RenderFormat::Svg, PAGED_FORMATS)
        .and_then(|format| wrap::check_packages(&packages).map(|_| format))
        .and_then(|format| request.width.as_deref().map(wrap::check_width).unwrap_or(Ok(())).map(|_| format))
        .and_then(|format| page_image(None, request.scale).map(|image| (format, image)));
    let (format, image) = match checked {
        Ok(choices) => choices,
        Err(message) => return bad_request(message),
    };
    let wrapped = wrap::snippet_document(&request.code, request.preamble.as_deref(), &packages, request.width.as_deref());
    render_job(&state, &caller, wrapped_job("snippet", wrapped, request.files), format, image).await
}

pub async fn check_document(State(state): State<AppState>, headers: HeaderMap, Json(request): Json<CheckRequest>) -> Response {
    let caller = match caller(&state, &headers).await {
        Ok(caller) => caller,
        Err(response) => return response,
    };
    if request.code.trim().is_empty() {
        return bad_request("code cannot be empty");
    }
    check_job(&state, &caller, SourceJob::document(Language::Latex, request.code, request.files)).await
}

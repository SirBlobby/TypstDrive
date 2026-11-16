use axum::{extract::State, http::HeaderMap, response::Response, Json};
use serde::Deserialize;

use crate::api::authentication::{authenticate, ApiScope};
use crate::api::errors::failure_response;
use crate::api::options::{bad_request, content_response, page_image, RenderFormat, ALL_FORMATS};
use crate::api::source_render::{render_job, render_output, Language, SourceJob};
use crate::compile::compiler::ProjectInput;
use crate::compile::render::PageImage;
use crate::AppState;

#[derive(Deserialize)]
pub struct RenderRequest {
    pub code: String,
    pub format: Option<String>,
    pub files: Option<Vec<InlineFile>>,
    pub page: Option<usize>,
    pub scale: Option<f32>,
}

#[derive(Deserialize)]
pub struct InlineFile {
    pub name: String,
    pub data: String,
}

pub async fn render_to_response(input: ProjectInput, format: RenderFormat, image: PageImage, source: Option<&str>) -> Response {
    match render_output(input, format, image).await {
        Ok(data) => content_response(format.content_type(), data),
        Err(failure) => failure_response(source, 0, failure),
    }
}

pub async fn render_handler(State(state): State<AppState>, headers: HeaderMap, Json(payload): Json<RenderRequest>) -> Response {
    let caller = match authenticate(&state, &headers, ApiScope::Render).await {
        Ok(caller) => caller,
        Err(response) => return response,
    };
    if payload.code.trim().is_empty() {
        return bad_request("code cannot be empty");
    }
    let format = match RenderFormat::choose(payload.format.as_deref(), RenderFormat::Pdf, ALL_FORMATS) {
        Ok(format) => format,
        Err(message) => return bad_request(message),
    };
    let image = match page_image(payload.page, payload.scale) {
        Ok(image) => image,
        Err(message) => return bad_request(message),
    };
    let job = SourceJob::document(Language::Typst, payload.code, payload.files);
    render_job(&state, &caller, job, format, image).await
}

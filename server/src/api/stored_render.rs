use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::api::authentication::{authenticate, ApiScope};
use crate::api::render::{invalid_format, render_to_response, RenderFormat};
use crate::compile::latex;
use crate::drive::versions::current_text;
use crate::sharing::access::{self, AccessRole};
use crate::AppState;

#[derive(Deserialize)]
pub struct StoredRenderRequest {
    pub format: String,
}

fn not_found(label: &str) -> Response {
    (StatusCode::NOT_FOUND, format!("{label} not found")).into_response()
}

pub async fn render_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(document_id): Path<String>,
    Json(payload): Json<StoredRenderRequest>,
) -> Response {
    let caller = match authenticate(&state, &headers, ApiScope::Render).await {
        Ok(caller) => caller,
        Err(response) => return response,
    };
    let Some(format) = RenderFormat::parse(&payload.format) else {
        return invalid_format();
    };

    let account = Some(caller.account_id);
    let document = match access::document_role(&state, &document_id, &account).await {
        Some((document, AccessRole::Owner)) => document,
        _ => return not_found("Document"),
    };

    let text = current_text(&state, &document.id).await.unwrap_or_default();
    let files = crate::drive::document_assets::compile_files(&state, &document, AccessRole::Owner, &account).await;
    let source = (!latex::is_latex(&document)).then_some(text.as_str());
    let input = latex::document_input(&document, text.clone(), files);
    render_to_response(input, format, source).await
}

pub async fn render_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    Json(payload): Json<StoredRenderRequest>,
) -> Response {
    let caller = match authenticate(&state, &headers, ApiScope::Render).await {
        Ok(caller) => caller,
        Err(response) => return response,
    };
    let Some(format) = RenderFormat::parse(&payload.format) else {
        return invalid_format();
    };

    let account = Some(caller.account_id);
    let project = match access::project_access(&state, &project_id, &account).await {
        Some((project, AccessRole::Owner)) => project,
        _ => return not_found("Project"),
    };

    let input = crate::drive::projects::assemble_project(&state, &project, HashMap::new(), crate::drive::projects::ProjectReader::Owner).await;
    render_to_response(input, format, None).await
}

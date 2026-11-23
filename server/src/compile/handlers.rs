use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::compile::compiler::{CompileFailure, DocumentStats, ProjectInput};
use crate::compile::latex;
use crate::compile::render::{self, RenderFailure};
use crate::compile::speaker_notes::Note;
use crate::sharing::access;
use crate::AppState;
use crate::db::sql;

#[derive(Deserialize)]
pub struct CompileRequest {
    #[serde(default)]
    pub text: Option<String>,
    pub document_id: Option<String>,
    pub project_id: Option<String>,
    #[serde(default)]
    pub files: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
pub struct CompileResponse {
    pub svgs: Option<Vec<String>>,
    pub errors: Option<Vec<Diagnostic>>,
    pub stats: Option<DocumentStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<Option<Note>>>,
}

#[derive(Serialize)]
pub struct Diagnostic {
    pub message: String,
    pub severity: String,
    pub from: Option<usize>,
    pub to: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

enum ThumbnailTarget {
    Document(String),
    Project(String),
}

struct CompileSource {
    input: ProjectInput,
    thumbnail_target: Option<ThumbnailTarget>,
}

fn map_diagnostics(diagnostics: CompileFailure) -> Vec<Diagnostic> {
    diagnostics
        .into_iter()
        .map(|(diagnostic, range)| Diagnostic {
            message: diagnostic.message.to_string(),
            severity: format!("{:?}", diagnostic.severity),
            from: range.as_ref().map(|range| range.start),
            to: range.as_ref().map(|range| range.end),
            file: None,
        })
        .collect()
}

fn single_error(message: &str) -> Vec<Diagnostic> {
    vec![Diagnostic { message: message.to_string(), severity: "Error".to_string(), from: None, to: None, file: None }]
}

fn failure_diagnostics(failure: RenderFailure) -> Vec<Diagnostic> {
    match failure {
        RenderFailure::Typst(diagnostics) => map_diagnostics(diagnostics),
        RenderFailure::Latex { entrypoint, problems } => problems
            .into_iter()
            .map(|problem| Diagnostic {
                message: problem.located_message(&entrypoint),
                severity: "Error".to_string(),
                from: problem.range.map(|range| range.0),
                to: problem.range.map(|range| range.1),
                file: problem.range.and(problem.file),
            })
            .collect(),
        RenderFailure::Unsupported(message) => single_error(&message),
        RenderFailure::LatexTurnedOff => single_error(crate::compile::latex_policy::TURNED_OFF_MESSAGE),
        RenderFailure::Stopped => single_error("Compilation stopped unexpectedly"),
    }
}

fn error_response(message: &str) -> Json<CompileResponse> {
    Json(CompileResponse { svgs: None, errors: Some(single_error(message)), stats: None, notes: None })
}

async fn resolve_source(state: &AppState, payload: &CompileRequest, user_id: &Option<String>) -> Option<CompileSource> {
    if let Some(project_id) = &payload.project_id {
        let (project, role) = access::project_access(state, project_id, user_id).await?;
        let overrides = if role.can_edit() { payload.files.clone().unwrap_or_default() } else { HashMap::new() };
        let reader = crate::drive::projects::ProjectReader::for_role(role, user_id);
        let input = crate::drive::projects::assemble_project(state, &project, overrides, reader).await;
        let thumbnail_target = role.can_edit().then(|| ThumbnailTarget::Project(project.id.clone()));
        return Some(CompileSource { input, thumbnail_target });
    }

    let Some(document_id) = &payload.document_id else {
        user_id.as_ref()?;
        return Some(CompileSource {
            input: ProjectInput::single(payload.text.clone().unwrap_or_default(), HashMap::new()),
            thumbnail_target: None,
        });
    };

    let (document, role) = access::document_role(state, document_id, user_id).await?;
    let files = crate::drive::document_assets::compile_files(state, &document, role, user_id).await;
    match (role.can_edit(), payload.text.clone()) {
        (true, Some(submitted_text)) => {
            let thumbnail_target = Some(ThumbnailTarget::Document(document.id.clone()));
            let input = latex::document_input(&document, submitted_text, files);
            Some(CompileSource { input, thumbnail_target })
        }
        _ => {
            let input = latex::document_input(&document, current_text(state, &document.id).await, files);
            Some(CompileSource { input, thumbnail_target: None })
        }
    }
}

/// The input `/api/compile` would build for this request, with the same access checks.
pub(crate) async fn compile_input(state: &AppState, payload: &CompileRequest, user_id: &Option<String>) -> Option<ProjectInput> {
    resolve_source(state, payload, user_id).await.map(|source| source.input)
}

async fn current_text(state: &AppState, document_id: &str) -> String {
    let room_id = crate::realtime::live_documents::document_room(document_id);
    match crate::realtime::live_documents::live_text(state, &room_id).await {
        Some(text) => text,
        None => crate::drive::documents::saved_text(state, document_id).await,
    }
}

async fn save_thumbnail(state: &AppState, target: ThumbnailTarget, thumbnail: &str) {
    let (statement, item_id) = match &target {
        ThumbnailTarget::Document(id) => ("UPDATE documents SET thumbnail_svg = $1 WHERE id = $2", id),
        ThumbnailTarget::Project(id) => ("UPDATE projects SET thumbnail_svg = $1 WHERE id = $2", id),
    };
    let _ = sqlx::query(sql(statement)).bind(thumbnail).bind(item_id).execute(&state.db).await;
}

pub async fn compile_handler(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<CompileRequest>,
) -> Json<CompileResponse> {
    let user_id = jar.get("session_user_id").map(|cookie| cookie.value().to_string());
    let Some(source) = resolve_source(&state, &payload, &user_id).await else {
        return error_response("Unauthorized");
    };

    match render::preview(source.input).await {
        Ok(preview) => {
            if let Some(target) = source.thumbnail_target {
                save_thumbnail(&state, target, &preview.thumbnail).await;
            }
            Json(CompileResponse { svgs: Some(preview.svgs), errors: None, stats: Some(preview.stats), notes: preview.notes })
        }
        Err(failure) => Json(CompileResponse { svgs: None, errors: Some(failure_diagnostics(failure)), stats: None, notes: None }),
    }
}

fn file_response(content_type: &'static str, bytes: Vec<u8>) -> Response {
    (StatusCode::OK, [(header::CONTENT_TYPE, content_type)], bytes).into_response()
}

pub async fn export_handler(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(format): Path<String>,
    Json(payload): Json<CompileRequest>,
) -> Response {
    if !matches!(format.as_str(), "pdf" | "png" | "svg" | "html") {
        return (StatusCode::NOT_FOUND, "Format not supported").into_response();
    }

    let user_id = jar.get("session_user_id").map(|cookie| cookie.value().to_string());
    let Some(source) = resolve_source(&state, &payload, &user_id).await else {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    };

    let input = source.input;
    let exported = match format.as_str() {
        "pdf" => render::pdf(input).await.map(|bytes| ("application/pdf", bytes)),
        "png" => render::png(input).await.map(|bytes| ("image/png", bytes)),
        "html" => render::html(input).await.map(|html| ("text/html; charset=utf-8", html.into_bytes())),
        _ => render::svg(input).await.map(|svg| ("image/svg+xml", svg.into_bytes())),
    };

    match exported {
        Ok((content_type, bytes)) => file_response(content_type, bytes),
        Err(failure) => {
            let summary = failure_diagnostics(failure).into_iter().map(|diagnostic| diagnostic.message).collect::<Vec<_>>().join("\n");
            (StatusCode::BAD_REQUEST, summary).into_response()
        }
    }
}

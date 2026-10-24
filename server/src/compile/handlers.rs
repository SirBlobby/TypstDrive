use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::compile::compiler::{run_blocking, CompileFailure, DocumentStats, ProjectInput};
use crate::compile::latex;
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
}

#[derive(Serialize)]
pub struct Diagnostic {
    pub message: String,
    pub severity: String,
    pub from: Option<usize>,
    pub to: Option<usize>,
}

enum ThumbnailTarget {
    Document(String),
    Project(String),
}

struct CompileSource {
    input: ProjectInput,
    thumbnail_target: Option<ThumbnailTarget>,
    generated_source: bool,
}

fn map_diagnostics(diagnostics: CompileFailure) -> Vec<Diagnostic> {
    diagnostics
        .into_iter()
        .map(|(diagnostic, range)| Diagnostic {
            message: diagnostic.message.to_string(),
            severity: format!("{:?}", diagnostic.severity),
            from: range.as_ref().map(|range| range.start),
            to: range.as_ref().map(|range| range.end),
        })
        .collect()
}

fn error_response(message: &str) -> Json<CompileResponse> {
    Json(CompileResponse {
        svgs: None,
        errors: Some(vec![Diagnostic {
            message: message.to_string(),
            severity: "Error".to_string(),
            from: None,
            to: None,
        }]),
        stats: None,
    })
}

async fn resolve_source(state: &AppState, payload: &CompileRequest, user_id: &Option<String>) -> Option<CompileSource> {
    if let Some(project_id) = &payload.project_id {
        let (project, role) = access::project_access(state, project_id, user_id).await?;
        let overrides = if role.can_edit() { payload.files.clone().unwrap_or_default() } else { HashMap::new() };
        let reader = crate::drive::projects::ProjectReader::for_role(role, user_id);
        let input = crate::drive::projects::assemble_project(state, &project, overrides, reader).await;
        let thumbnail_target = role.can_edit().then(|| ThumbnailTarget::Project(project.id.clone()));
        let generated_source = latex::is_latex_path(&input.entrypoint);
        return Some(CompileSource { input, thumbnail_target, generated_source });
    }

    let submitted_text = payload.text.clone().unwrap_or_default();
    let Some(document_id) = &payload.document_id else {
        user_id.as_ref()?;
        return Some(CompileSource {
            input: ProjectInput::single(submitted_text, HashMap::new()),
            thumbnail_target: None,
            generated_source: false,
        });
    };

    let (document, role) = access::document_role(state, document_id, user_id).await?;
    let files = crate::drive::document_assets::compile_files(state, &document, role, user_id).await;
    let generated_source = latex::is_latex(&document);
    if role.can_edit() {
        let thumbnail_target = Some(ThumbnailTarget::Document(document.id.clone()));
        let input = latex::document_input(&document, submitted_text, files);
        Some(CompileSource { input, thumbnail_target, generated_source })
    } else {
        let saved_text = crate::drive::documents::saved_text(state, &document.id).await;
        let input = latex::document_input(&document, saved_text, files);
        Some(CompileSource { input, thumbnail_target: None, generated_source })
    }
}

fn without_ranges(mut diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    for diagnostic in &mut diagnostics {
        diagnostic.from = None;
        diagnostic.to = None;
    }
    diagnostics
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

    let generated_source = source.generated_source;
    let Some(result) = run_blocking(move |compiler| compiler.compile_svg(source.input)).await else {
        return error_response("Compilation stopped unexpectedly");
    };

    match result {
        Ok((svgs, thumbnail, stats)) => {
            if let Some(target) = source.thumbnail_target {
                save_thumbnail(&state, target, &thumbnail).await;
            }
            Json(CompileResponse { svgs: Some(svgs), errors: None, stats: Some(stats) })
        }
        Err(diagnostics) => {
            let mapped = map_diagnostics(diagnostics);
            let errors = if generated_source { without_ranges(mapped) } else { mapped };
            Json(CompileResponse { svgs: None, errors: Some(errors), stats: None })
        }
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
    if !matches!(format.as_str(), "pdf" | "png" | "svg") {
        return (StatusCode::NOT_FOUND, "Format not supported").into_response();
    }

    let user_id = jar.get("session_user_id").map(|cookie| cookie.value().to_string());
    let Some(source) = resolve_source(&state, &payload, &user_id).await else {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    };

    let input = source.input;
    let exported = run_blocking(move |compiler| match format.as_str() {
        "pdf" => compiler.export_pdf(input).map(|bytes| ("application/pdf", bytes)),
        "png" => compiler.export_png(input).map(|bytes| ("image/png", bytes)),
        _ => compiler.compile_svg(input).map(|(svgs, _, _)| ("image/svg+xml", svgs.join("\n").into_bytes())),
    })
    .await;

    match exported {
        Some(Ok((content_type, bytes))) => file_response(content_type, bytes),
        _ => (StatusCode::BAD_REQUEST, "Compilation failed").into_response(),
    }
}

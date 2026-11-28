use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::compile::compiler::{CompileFailure, DocumentStats, ProjectInput};
use crate::compile::latex;
use crate::compile::{preview_store, warm_up};
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
    #[serde(default)]
    pub known_pages: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct CompileResponse {
    pub svgs: Option<Vec<Option<String>>>,
    pub errors: Option<Vec<Diagnostic>>,
    pub stats: Option<DocumentStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<Option<Note>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_hashes: Option<Vec<String>>,
}

pub const BUSY_MESSAGE: &str = "The server is busy compiling. Try again in a moment.";
pub const TIMED_OUT_MESSAGE: &str = "Compiling took too long and was stopped. Check the document for loops that never end.";

const MAX_KNOWN_PAGES: usize = 20_000;

fn page_hash(svg: &str) -> String {
    let mut hash = crate::hashing::sha256_hex(svg.as_bytes());
    hash.truncate(32);
    hash
}

fn is_page_hash(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

async fn preview_response(svgs: Vec<String>, stats: DocumentStats, notes: Option<Vec<Option<Note>>>, known_pages: Option<Vec<String>>) -> Json<CompileResponse> {
    let Some(mut known) = known_pages else {
        let svgs = svgs.into_iter().map(Some).collect();
        return Json(CompileResponse { svgs: Some(svgs), errors: None, stats: Some(stats), notes, page_hashes: None });
    };
    known.retain(|hash| is_page_hash(hash));
    known.truncate(MAX_KNOWN_PAGES);
    let hashed = tokio::task::spawn_blocking(move || {
        let known: HashSet<String> = known.into_iter().collect();
        let hashes: Vec<String> = svgs.iter().map(|svg| page_hash(svg)).collect();
        let svgs: Vec<Option<String>> = svgs.into_iter().zip(&hashes).map(|(svg, hash)| (!known.contains(hash)).then_some(svg)).collect();
        (svgs, hashes)
    })
    .await;
    match hashed {
        Ok((svgs, hashes)) => Json(CompileResponse { svgs: Some(svgs), errors: None, stats: Some(stats), notes, page_hashes: Some(hashes) }),
        Err(_) => error_response("Compilation stopped unexpectedly"),
    }
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

pub(crate) struct CompileSource {
    pub input: ProjectInput,
    thumbnail_target: Option<ThumbnailTarget>,
    pub cache_target: Option<String>,
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
        RenderFailure::Busy => single_error(BUSY_MESSAGE),
        RenderFailure::TimedOut => single_error(TIMED_OUT_MESSAGE),
    }
}

fn error_response(message: &str) -> Json<CompileResponse> {
    Json(CompileResponse { svgs: None, errors: Some(single_error(message)), stats: None, notes: None, page_hashes: None })
}

pub(crate) async fn resolve_source(state: &AppState, payload: &CompileRequest, user_id: &Option<String>) -> Option<CompileSource> {
    if let Some(project_id) = &payload.project_id {
        let (project, role) = access::project_access(state, project_id, user_id).await?;
        let overrides = if role.can_edit() { payload.files.clone().unwrap_or_default() } else { HashMap::new() };
        let reader = crate::drive::projects::ProjectReader::for_role(role, user_id);
        let input = crate::drive::projects::assemble_project(state, &project, overrides, reader).await;
        let thumbnail_target = role.can_edit().then(|| ThumbnailTarget::Project(project.id.clone()));
        let cache_target = Some(format!("project:{}", project.id));
        return Some(CompileSource { input, thumbnail_target, cache_target });
    }

    let Some(document_id) = &payload.document_id else {
        user_id.as_ref()?;
        return Some(CompileSource {
            input: ProjectInput::single(payload.text.clone().unwrap_or_default(), HashMap::new()),
            thumbnail_target: None,
            cache_target: None,
        });
    };

    let (document, role) = access::document_role(state, document_id, user_id).await?;
    let files = crate::drive::document_assets::compile_files(state, &document, role, user_id).await;
    let cache_target = Some(format!("document:{}", document.id));
    match (role.can_edit(), payload.text.clone()) {
        (true, Some(submitted_text)) => {
            let thumbnail_target = Some(ThumbnailTarget::Document(document.id.clone()));
            let input = latex::document_input(&document, submitted_text, files);
            Some(CompileSource { input, thumbnail_target, cache_target })
        }
        _ => {
            let input = latex::document_input(&document, current_text(state, &document.id).await, files);
            Some(CompileSource { input, thumbnail_target: None, cache_target })
        }
    }
}

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

    let known_pages = payload.known_pages;
    let CompileSource { input, thumbnail_target, cache_target } = source;
    let cache_target = cache_target.filter(|_| !render::is_latex(&input));
    let warm = cache_target.as_deref().is_some_and(warm_up::is_warm);
    let (input, key) = match &cache_target {
        Some(_) if !warm => match tokio::task::spawn_blocking(move || {
            let key = preview_store::key(&input);
            (input, key)
        })
        .await
        {
            Ok(keyed) => keyed,
            Err(_) => return error_response("Compilation stopped unexpectedly"),
        },
        _ => (input, None),
    };

    if let (Some(target), Some(key)) = (&cache_target, &key) {
        let lookup = key.clone();
        if let Ok(Some(stored)) = tokio::task::spawn_blocking(move || preview_store::load(&lookup)).await {
            if let (Some(thumbnail_target), Some(first_page)) = (thumbnail_target, stored.svgs.first()) {
                save_thumbnail(&state, thumbnail_target, first_page).await;
            }
            warm_up::refresh(input, key.clone(), target.clone());
            return preview_response(stored.svgs, stored.stats, stored.notes, known_pages).await;
        }
    }

    let input_to_key = (warm && preview_store::is_enabled()).then(|| input.clone());
    match render::preview(input).await {
        Ok(preview) => {
            if let Some(target) = &cache_target {
                warm_up::note_compiled(target);
                if let Some(today) = preview.outside_dependencies {
                    let stored = StoredParts { target: target.clone(), svgs: preview.svgs.clone(), stats: preview.stats.clone(), notes: preview.notes.clone(), today };
                    store_in_background(key, input_to_key, stored);
                }
            }
            if let Some(target) = thumbnail_target {
                save_thumbnail(&state, target, &preview.thumbnail).await;
            }
            preview_response(preview.svgs, preview.stats, preview.notes, known_pages).await
        }
        Err(failure) => Json(CompileResponse { svgs: None, errors: Some(failure_diagnostics(failure)), stats: None, notes: None, page_hashes: None }),
    }
}

struct StoredParts {
    target: String,
    svgs: Vec<String>,
    stats: DocumentStats,
    notes: Option<Vec<Option<Note>>>,
    today: Vec<crate::compile::world::TodayAnswer>,
}

fn store_in_background(key: Option<String>, input_to_key: Option<ProjectInput>, parts: StoredParts) {
    tokio::task::spawn_blocking(move || {
        let Some(key) = key.or_else(|| input_to_key.as_ref().and_then(preview_store::key)) else {
            return;
        };
        preview_store::save(&key, &parts.target, &parts.svgs, &parts.stats, &parts.notes, parts.today);
    });
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

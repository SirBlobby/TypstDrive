use std::collections::HashSet;
use std::sync::atomic::{AtomicI64, Ordering};

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::api::authentication::{authenticate, ApiScope};
use crate::compile::compiler::{run_blocking, CompileFailure, ProjectInput};
use crate::db::sql;
use crate::drive::file_data::account_files;
use crate::AppState;

const CACHE_MAX_AGE_SECONDS: i64 = 3600;
const CACHE_PRUNE_INTERVAL_SECONDS: i64 = 300;

static LAST_CACHE_PRUNE: AtomicI64 = AtomicI64::new(0);

#[derive(Clone, Copy)]
pub enum RenderFormat {
    Png,
    Pdf,
    Html,
}

impl RenderFormat {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "png" => Some(Self::Png),
            "pdf" => Some(Self::Pdf),
            "html" => Some(Self::Html),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Pdf => "pdf",
            Self::Html => "html",
        }
    }

    fn content_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Pdf => "application/pdf",
            Self::Html => "text/html; charset=utf-8",
        }
    }
}

#[derive(Deserialize)]
pub struct RenderRequest {
    pub code: String,
    pub format: String,
    pub files: Option<Vec<InlineFile>>,
}

#[derive(Deserialize)]
pub struct InlineFile {
    pub name: String,
    pub data: String,
}

#[derive(Serialize)]
struct CompileErrorDetail {
    message: String,
    severity: String,
    line: Option<usize>,
    column: Option<usize>,
}

#[derive(Serialize)]
struct CompileErrorResponse {
    error: String,
    details: Vec<CompileErrorDetail>,
}

pub fn invalid_format() -> Response {
    (StatusCode::BAD_REQUEST, "Invalid format. Must be 'png', 'pdf', or 'html'").into_response()
}

fn line_and_column(code: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    for (index, character) in code.char_indices() {
        if index >= offset {
            break;
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

fn compile_error_response(source: Option<&str>, diagnostics: CompileFailure) -> Response {
    let details: Vec<CompileErrorDetail> = diagnostics
        .into_iter()
        .map(|(diagnostic, range)| {
            let position = match (source, range) {
                (Some(code), Some(range)) => Some(line_and_column(code, range.start)),
                _ => None,
            };
            CompileErrorDetail {
                message: diagnostic.message.to_string(),
                severity: format!("{:?}", diagnostic.severity).to_lowercase(),
                line: position.map(|(line, _)| line),
                column: position.map(|(_, column)| column),
            }
        })
        .collect();

    let summary = details
        .iter()
        .map(|detail| match (detail.line, detail.column) {
            (Some(line), Some(column)) => format!("{} (line {line}, column {column})", detail.message),
            _ => detail.message.clone(),
        })
        .collect::<Vec<_>>()
        .join("; ");

    let error = if summary.is_empty() {
        "Typst compilation failed.".to_string()
    } else {
        format!("Typst compilation failed: {summary}")
    };
    (StatusCode::UNPROCESSABLE_ENTITY, Json(CompileErrorResponse { error, details })).into_response()
}

fn rendered_response(format: RenderFormat, data: Vec<u8>) -> Response {
    (StatusCode::OK, [(header::CONTENT_TYPE, format.content_type())], data).into_response()
}

fn compiler_stopped() -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, "Compilation stopped unexpectedly").into_response()
}

async fn render_input(input: ProjectInput, format: RenderFormat) -> Option<Result<Vec<u8>, CompileFailure>> {
    run_blocking(move |compiler| match format {
        RenderFormat::Pdf => compiler.export_pdf(input),
        RenderFormat::Png => compiler.export_png(input),
        RenderFormat::Html => compiler.export_html(input).map(String::into_bytes),
    })
    .await
}

pub async fn render_to_response(input: ProjectInput, format: RenderFormat, source: Option<&str>) -> Response {
    match render_input(input, format).await {
        Some(Ok(data)) => rendered_response(format, data),
        Some(Err(diagnostics)) => compile_error_response(source, diagnostics),
        None => compiler_stopped(),
    }
}

fn compute_cache_key(account_id: &str, format: RenderFormat, code: &str, files: &Option<Vec<InlineFile>>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(b"\x00");
    hasher.update(format.as_str().as_bytes());
    hasher.update(b"\x00");
    hasher.update(code.as_bytes());
    if let Some(files) = files {
        let mut pairs: Vec<_> = files.iter().map(|file| (file.name.as_str(), file.data.as_str())).collect();
        pairs.sort_by_key(|(name, _)| *name);
        for (name, data) in pairs {
            hasher.update(b"\x01");
            hasher.update(name.as_bytes());
            hasher.update(data.as_bytes());
        }
    }
    crate::hashing::to_hex(hasher.finalize().as_slice())
}

async fn cached_render(state: &AppState, cache_key: &str, format: RenderFormat) -> Option<Vec<u8>> {
    let (data, created_at) = sqlx::query_as::<_, (Vec<u8>, String)>(sql(
        "SELECT data, created_at FROM api_render_cache WHERE content_hash = $1 AND format = $2",
    ))
    .bind(cache_key)
    .bind(format.as_str())
    .fetch_optional(&state.db)
    .await
    .ok()??;

    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at).ok()?;
    let age = chrono::Utc::now().signed_duration_since(created_at.with_timezone(&chrono::Utc));
    (age.num_seconds() < CACHE_MAX_AGE_SECONDS).then_some(data)
}

async fn store_render(state: &AppState, cache_key: &str, format: RenderFormat, data: &[u8]) {
    let _ = sqlx::query(sql(
        "INSERT INTO api_render_cache (id, content_hash, format, data, created_at) VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (content_hash) DO UPDATE SET format = excluded.format, data = excluded.data, created_at = excluded.created_at",
    ))
    .bind(Uuid::new_v4().to_string())
    .bind(cache_key)
    .bind(format.as_str())
    .bind(data.to_vec())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&state.db)
    .await;
}

async fn prune_cache(state: &AppState) {
    let now = chrono::Utc::now();
    let last_prune = LAST_CACHE_PRUNE.load(Ordering::Relaxed);
    if now.timestamp() - last_prune < CACHE_PRUNE_INTERVAL_SECONDS
        || LAST_CACHE_PRUNE.compare_exchange(last_prune, now.timestamp(), Ordering::Relaxed, Ordering::Relaxed).is_err()
    {
        return;
    }
    let cutoff = (now - chrono::TimeDelta::seconds(CACHE_MAX_AGE_SECONDS)).to_rfc3339();
    let _ = sqlx::query(sql("DELETE FROM api_render_cache WHERE created_at < $1"))
        .bind(cutoff)
        .execute(&state.db)
        .await;
}

pub async fn render_handler(State(state): State<AppState>, headers: HeaderMap, Json(payload): Json<RenderRequest>) -> Response {
    let caller = match authenticate(&state, &headers, ApiScope::Render).await {
        Ok(caller) => caller,
        Err(response) => return response,
    };
    let Some(format) = RenderFormat::parse(&payload.format) else {
        return invalid_format();
    };
    if payload.code.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "code cannot be empty").into_response();
    }

    prune_cache(&state).await;
    let cache_key = compute_cache_key(&caller.account_id, format, &payload.code, &payload.files);
    if let Some(data) = cached_render(&state, &cache_key, format).await {
        return rendered_response(format, data);
    }

    let mut files = account_files(&state, &caller.account_id, &[payload.code.as_str()], &HashSet::new()).await;
    for inline_file in payload.files.iter().flatten() {
        if let Ok(decoded) = BASE64.decode(&inline_file.data) {
            files.insert(inline_file.name.clone(), decoded);
        }
    }

    let input = ProjectInput::single(payload.code.clone(), files);
    match render_input(input, format).await {
        Some(Ok(data)) => {
            store_render(&state, &cache_key, format, &data).await;
            rendered_response(format, data)
        }
        Some(Err(diagnostics)) => compile_error_response(Some(payload.code.as_str()), diagnostics),
        None => compiler_stopped(),
    }
}

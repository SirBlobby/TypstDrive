use std::process::Stdio;

use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::compile::handlers::CompileRequest;
use crate::sharing::access;
use crate::AppState;

const EXPORT_FORMATS: [&str; 4] = ["docx", "latex", "markdown", "html"];

fn unauthorized() -> Response {
    (StatusCode::UNAUTHORIZED, "Sign in to convert documents").into_response()
}

fn export_content_type(format: &str) -> &'static str {
    match format {
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "html" => "text/html",
        "latex" => "application/x-latex",
        "markdown" => "text/markdown",
        _ => "application/octet-stream",
    }
}

fn import_format(file_name: &str) -> &'static str {
    let lower = file_name.to_lowercase();
    if lower.ends_with(".docx") {
        "docx"
    } else if lower.ends_with(".tex") {
        "latex"
    } else if lower.ends_with(".html") {
        "html"
    } else {
        "markdown"
    }
}

async fn may_convert(state: &AppState, payload: &CompileRequest, user_id: &Option<String>) -> bool {
    if user_id.is_some() {
        return true;
    }
    if let Some(document_id) = &payload.document_id {
        return access::document_role(state, document_id, user_id).await.is_some();
    }
    if let Some(project_id) = &payload.project_id {
        return access::project_access(state, project_id, user_id).await.is_some();
    }
    false
}

async fn run_pandoc(from: &str, to: &str, input: Vec<u8>) -> Result<Vec<u8>, Response> {
    let mut child = Command::new("pandoc")
        .arg("--sandbox")
        .arg("-f")
        .arg(from)
        .arg("-t")
        .arg(to)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to start pandoc: {error}")).into_response())?;

    if let Some(mut stdin) = child.stdin.take() {
        tokio::spawn(async move {
            let _ = stdin.write_all(&input).await;
        });
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pandoc failed: {error}")).into_response())?;

    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr);
        return Err((StatusCode::BAD_REQUEST, format!("Pandoc error: {message}")).into_response());
    }
    Ok(output.stdout)
}

pub async fn pandoc_export_handler(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(format): Path<String>,
    Json(payload): Json<CompileRequest>,
) -> Response {
    if !EXPORT_FORMATS.contains(&format.as_str()) {
        return (StatusCode::BAD_REQUEST, "Unsupported format").into_response();
    }

    let user_id = jar.get("session_user_id").map(|cookie| cookie.value().to_string());
    if !may_convert(&state, &payload, &user_id).await {
        return unauthorized();
    }

    let text = payload.text.unwrap_or_default();
    match run_pandoc("typst", &format, text.into_bytes()).await {
        Ok(bytes) => (StatusCode::OK, [(header::CONTENT_TYPE, export_content_type(&format))], bytes).into_response(),
        Err(response) => response,
    }
}

pub async fn pandoc_import_handler(jar: SignedCookieJar, mut multipart: Multipart) -> Response {
    if jar.get("session_user_id").is_none() {
        return unauthorized();
    }

    let mut file_data = Vec::new();
    let mut source_format = "markdown";

    if let Ok(Some(field)) = multipart.next_field().await {
        if let Some(file_name) = field.file_name() {
            source_format = import_format(file_name);
        }
        if let Ok(bytes) = field.bytes().await {
            file_data = bytes.to_vec();
        }
    }

    if file_data.is_empty() {
        return (StatusCode::BAD_REQUEST, "No file uploaded").into_response();
    }

    match run_pandoc(source_format, "typst", file_data).await {
        Ok(bytes) => (StatusCode::OK, [(header::CONTENT_TYPE, "text/plain; charset=utf-8")], bytes).into_response(),
        Err(response) => response,
    }
}

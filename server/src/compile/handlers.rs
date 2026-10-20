use axum::{
    extract::{Path, State, Multipart},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use futures_util::stream::StreamExt;
use crate::sharing::access::{self, Resource};
use crate::AppState;
use crate::drive::projects::decode_text_blob;

#[derive(Deserialize)]
pub struct CompileRequest {
    #[serde(default)]
    pub text: Option<String>,
    pub document_id: Option<String>,
    pub project_id: Option<String>,
    #[serde(default)]
    pub files: Option<std::collections::HashMap<String, String>>,
}

use crate::compile::compiler::{DocumentStats, ProjectInput};

fn map_diagnostics(
    diags: Vec<(typst::diag::SourceDiagnostic, Option<std::ops::Range<usize>>)>,
) -> Vec<Diagnostic> {
    diags
        .into_iter()
        .map(|(d, range)| Diagnostic {
            message: d.message.to_string(),
            severity: format!("{:?}", d.severity),
            from: range.as_ref().map(|r| r.start),
            to: range.as_ref().map(|r| r.end),
        })
        .collect()
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

fn unauthorized_compile() -> Json<CompileResponse> {
    Json(CompileResponse {
        svgs: None,
        errors: Some(vec![Diagnostic {
            message: "Unauthorized".to_string(),
            severity: "Error".to_string(),
            from: None,
            to: None,
        }]),
        stats: None,
    })
}

async fn document_source(
    state: &AppState,
    payload: &CompileRequest,
    user_id: &Option<String>,
) -> Result<(String, HashMap<String, Vec<u8>>, bool), ()> {
    let submitted_text = payload.text.clone().unwrap_or_default();
    let Some(document_id) = &payload.document_id else {
        return Ok((submitted_text, HashMap::new(), false));
    };

    let (document, role) = access::document_role(state, document_id, user_id).await.ok_or(())?;
    let files_map = crate::drive::document_assets::compile_files(state, &document, role, user_id).await;
    if role.can_edit() {
        Ok((submitted_text, files_map, true))
    } else {
        let saved_text = decode_text_blob(document.content.as_deref().unwrap_or_default());
        Ok((saved_text, files_map, false))
    }
}

pub async fn compile_handler(
    State(state): State<AppState>,
    jar: axum_extra::extract::cookie::SignedCookieJar,
    Json(payload): Json<CompileRequest>,
) -> impl IntoResponse {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());

    if let Some(project_id) = &payload.project_id {
        let Some((project, role)) = access::project_access(&state, project_id, &user_id_opt).await else {
            return unauthorized_compile();
        };

        let overrides = if role.can_edit() { payload.files.clone().unwrap_or_default() } else { HashMap::new() };
        let input = crate::drive::projects::assemble_project(&state, &project, overrides).await;

        let compiler = state.compiler.lock().await;
        let result = compiler.compile_svg(input);
        drop(compiler);

        return match result {
            Ok((svgs, thumbnail, stats)) => {
                if role.can_edit() {
                    let _ = sqlx::query("UPDATE projects SET thumbnail_svg = $1 WHERE id = $2")
                        .bind(&thumbnail)
                        .bind(&project.id)
                        .execute(&state.db)
                        .await;
                }
                Json(CompileResponse {
                    svgs: Some(svgs),
                    errors: None,
                    stats: Some(stats),
                })
            }
            Err(diags) => Json(CompileResponse {
                svgs: None,
                errors: Some(map_diagnostics(diags)),
                stats: None,
            }),
        };
    }

    let Ok((source_text, files_map, can_save_thumbnail)) = document_source(&state, &payload, &user_id_opt).await else {
        return unauthorized_compile();
    };

    let compiler = state.compiler.lock().await;
    let result = compiler.compile_svg(ProjectInput::single(source_text, files_map));
    drop(compiler);

    match result {
        Ok((svgs, thumbnail, stats)) => {
            if let (Some(doc_id), true) = (&payload.document_id, can_save_thumbnail) {
                let _ = sqlx::query("UPDATE documents SET thumbnail_svg = $1 WHERE id = $2")
                    .bind(&thumbnail)
                    .bind(doc_id)
                    .execute(&state.db)
                    .await;
            }

            Json(CompileResponse {
                svgs: Some(svgs),
                errors: None,
                stats: Some(stats),
            })
        }
        Err(diags) => Json(CompileResponse {
            svgs: None,
            errors: Some(map_diagnostics(diags)),
            stats: None,
        }),
    }
}

pub async fn export_handler(
    State(state): State<AppState>,
    jar: axum_extra::extract::cookie::SignedCookieJar,
    Path(format): Path<String>,
    Json(payload): Json<CompileRequest>,
) -> impl IntoResponse {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());

    let input = if let Some(project_id) = &payload.project_id {
        match access::project_access(&state, project_id, &user_id_opt).await {
            Some((project, role)) => {
                let overrides = if role.can_edit() { payload.files.clone().unwrap_or_default() } else { HashMap::new() };
                crate::drive::projects::assemble_project(&state, &project, overrides).await
            }
            None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
        }
    } else {
        match document_source(&state, &payload, &user_id_opt).await {
            Ok((source_text, files_map, _)) => ProjectInput::single(source_text, files_map),
            Err(()) => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
        }
    };

    let compiler = state.compiler.lock().await;

    match format.as_str() {
        "pdf" => match compiler.export_pdf(input) {
            Ok(bytes) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/pdf")],
                bytes,
            )
                .into_response(),
            Err(_) => (StatusCode::BAD_REQUEST, "Compilation failed").into_response(),
        },
        "png" => match compiler.export_png(input) {
            Ok(bytes) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "image/png")],
                bytes,
            )
                .into_response(),
            Err(_) => (StatusCode::BAD_REQUEST, "Compilation failed").into_response(),
        },
        "svg" => match compiler.compile_svg(input) {
            Ok((svgs, _, _)) => {
                let mut combined = String::new();
                for svg in svgs {
                    combined.push_str(&svg);
                    combined.push('\n');
                }
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "image/svg+xml")],
                    combined.into_bytes(),
                )
                    .into_response()
            }
            Err(_) => (StatusCode::BAD_REQUEST, "Compilation failed").into_response(),
        },
        _ => (StatusCode::NOT_FOUND, "Format not supported").into_response(),
    }
}

use std::process::Stdio;
use tokio::process::Command;

pub async fn pandoc_export_handler(
    Path(format): Path<String>,
    Json(payload): Json<CompileRequest>,
) -> impl IntoResponse {
    let supported_formats = ["docx", "latex", "markdown", "html"];
    if !supported_formats.contains(&format.as_str()) {
        return (StatusCode::BAD_REQUEST, "Unsupported format").into_response();
    }

    let _ext = match format.as_str() {
        "latex" => "tex",
        "markdown" => "md",
        f => f,
    };

    let mut child = match Command::new("pandoc")
        .arg("-f")
        .arg("typst")
        .arg("-t")
        .arg(&format)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to start pandoc: {}", e)).into_response(),
    };

    let mut stdin = child.stdin.take().unwrap();
    let text = payload.text.clone().unwrap_or_default();
    tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        let _ = stdin.write_all(text.as_bytes()).await;
    });

    let output = match child.wait_with_output().await {
        Ok(o) => o,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Pandoc failed: {}", e)).into_response(),
    };

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return (StatusCode::BAD_REQUEST, format!("Pandoc error: {}", err)).into_response();
    }

    let content_type = match format.as_str() {
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "html" => "text/html",
        "latex" => "application/x-latex",
        "markdown" => "text/markdown",
        _ => "application/octet-stream",
    };

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, content_type)],
        output.stdout,
    )
        .into_response()
}

pub async fn pandoc_import_handler(
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut file_data = Vec::new();
    let mut file_ext = String::new();

    if let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if let Some(file_name) = field.file_name() {
            if file_name.ends_with(".docx") {
                file_ext = "docx".to_string();
            } else if file_name.ends_with(".tex") {
                file_ext = "latex".to_string();
            } else if file_name.ends_with(".md") {
                file_ext = "markdown".to_string();
            } else if file_name.ends_with(".html") {
                file_ext = "html".to_string();
            } else {
                file_ext = "markdown".to_string();
            }
        }
        if let Ok(bytes) = field.bytes().await {
            file_data = bytes.to_vec();
        }
    }

    if file_data.is_empty() {
        return (StatusCode::BAD_REQUEST, "No file uploaded").into_response();
    }

    let mut child = match Command::new("pandoc")
        .arg("-f")
        .arg(&file_ext)
        .arg("-t")
        .arg("typst")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to start pandoc: {}", e)).into_response(),
    };

    let mut stdin = child.stdin.take().unwrap();
    tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        let _ = stdin.write_all(&file_data).await;
    });

    let output = match child.wait_with_output().await {
        Ok(o) => o,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Pandoc failed: {}", e)).into_response(),
    };

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return (StatusCode::BAD_REQUEST, format!("Pandoc error: {}", err)).into_response();
    }

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        output.stdout,
    )
        .into_response()
}

pub async fn lsp_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    Path(id): Path<String>,
    State(state): State<AppState>,
    jar: axum_extra::extract::cookie::SignedCookieJar,
) -> impl IntoResponse {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());

    let (doc, role) = match access::document_role(&state, &id, &user_id_opt).await {
        Some((doc, role)) if role.can_edit() => (doc, role),
        Some(_) => return access::forbidden().into_response(),
        None => return (StatusCode::NOT_FOUND, "Document not found").into_response(),
    };

    let files_map = crate::drive::document_assets::compile_files(&state, &doc, role, &user_id_opt).await;

    ws.on_upgrade(move |socket| async move {
        use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};

        let session = access::register_session(&state, Resource::Document(id.clone()), user_id_opt, None, true).await;
        use tokio::process::Command;
        use std::process::Stdio;

        let temp_dir = tempfile::tempdir().unwrap();

        for (name, data) in files_map {
            let is_safe_name = std::path::Path::new(&name)
                .components()
                .all(|part| matches!(part, std::path::Component::Normal(_)));
            if !is_safe_name {
                continue;
            }
            let path = temp_dir.path().join(&name);
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&path, data);
        }

        let mut child = Command::new("tinymist")
            .arg("lsp")
            .arg("--font-path")
            .arg(temp_dir.path())
            .current_dir(temp_dir.path())
            .env("RUST_LOG", "warn")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .expect("Failed to start tinymist lsp");

        let mut stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let mut stdout_reader = BufReader::new(stdout);

        let (mut ws_tx, mut ws_rx) = socket.split();

        let root_uri = format!("file://{}", temp_dir.path().display());
        let init_msg = serde_json::json!({
            "type": "init",
            "rootUri": root_uri
        });
        use futures_util::SinkExt;
        let _ = ws_tx.send(axum::extract::ws::Message::Text(init_msg.to_string().into())).await;

        let mut ws_to_lsp = tokio::spawn(async move {
            while let Some(Ok(axum::extract::ws::Message::Text(msg))) = ws_rx.next().await {
                let content_length = format!("Content-Length: {}\r\n\r\n", msg.len());
                if stdin.write_all(content_length.as_bytes()).await.is_err() {
                    break;
                }
                if stdin.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }
            }
        });

        let mut lsp_to_ws = tokio::spawn(async move {
            loop {
                let mut content_length = 0;
                let mut header = String::new();
                loop {
                    let mut char_buf = [0; 1];
                    if stdout_reader.read_exact(&mut char_buf).await.is_err() {
                        return;
                    }
                    header.push(char_buf[0] as char);
                    if header.ends_with("\r\n\r\n") {
                        break;
                    }
                }

                for line in header.split("\r\n") {
                    if line.starts_with("Content-Length: ") {
                        if let Ok(len) = line["Content-Length: ".len()..].trim().parse::<usize>() {
                            content_length = len;
                        }
                    }
                }

                if content_length == 0 { continue; }

                let mut body = vec![0; content_length];
                if stdout_reader.read_exact(&mut body).await.is_err() {
                    break;
                }

                if let Ok(text) = String::from_utf8(body) {
                    use futures_util::SinkExt;
                    if ws_tx.send(axum::extract::ws::Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
            }
        });

        tokio::select! {
            _ = &mut ws_to_lsp => {}
            _ = &mut lsp_to_ws => {}
            _ = child.wait() => {}
            _ = session.revoked => {}
        }

        ws_to_lsp.abort();
        lsp_to_ws.abort();
        let _ = child.kill().await;
        access::unregister_session(&state, &session.resource, session.session_id).await;
    })
}

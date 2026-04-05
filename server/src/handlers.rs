use axum::{
    extract::{Path, State, Multipart},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use yrs_axum::ws::AxumSink;
use yrs_axum::broadcast::BroadcastGroup;
use yrs::sync::Awareness;
use yrs::{Doc, ReadTxn, Transact, Update};
use yrs::updates::decoder::Decode;
use futures_util::stream::{StreamExt, Stream};
use crate::AppState;
use crate::models::Document;

pub struct ViewerFilterStream {
    inner: futures_util::stream::SplitStream<axum::extract::ws::WebSocket>,
    is_viewer: bool,
}

impl Stream for ViewerFilterStream {
    type Item = Result<Vec<u8>, yrs::sync::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            match futures_util::ready!(std::pin::Pin::new(&mut self.inner).poll_next(cx)) {
                Some(Ok(msg)) => {
                    if let axum::extract::ws::Message::Binary(bytes) = msg {
                        if self.is_viewer && !bytes.is_empty() && bytes[0] == 0 && bytes.len() > 1 && bytes[1] == 2 {
                            continue; // Skip updates
                        }
                        return std::task::Poll::Ready(Some(Ok(bytes.to_vec())));
                    } else if let axum::extract::ws::Message::Close(_) = msg {
                        return std::task::Poll::Ready(None);
                    }
                    continue;
                }
                Some(Err(e)) => return std::task::Poll::Ready(Some(Err(yrs::sync::Error::Other(Box::new(e))))),
                None => return std::task::Poll::Ready(None),
            }
        }
    }
}

#[derive(Deserialize)]
pub struct CompileRequest {
    pub text: String,
    pub document_id: Option<String>,
}

#[derive(Serialize)]
pub struct CompileResponse {
    pub svgs: Option<Vec<String>>,
    pub errors: Option<Vec<Diagnostic>>,
}

#[derive(Serialize)]
pub struct Diagnostic {
    pub message: String,
    pub severity: String,
}

pub async fn yjs_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    Path(id): Path<String>,
    State(state): State<AppState>,
    jar: axum_extra::extract::cookie::SignedCookieJar,
) -> impl IntoResponse {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());
    
    let doc_info = sqlx::query_as::<_, Document>(
        "SELECT id, owner_id, folder_id, title, content, thumbnail_svg, public_role, created_at, updated_at FROM documents WHERE id = $1"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await;

    let mut is_viewer = true;
    if let Ok(Some(ref d)) = doc_info {
        if let Some(uid) = &user_id_opt {
            if &d.owner_id == uid {
                is_viewer = false;
            } else if let Ok(Some(_)) = sqlx::query_as::<_, (String,)>("SELECT role FROM collaborators WHERE document_id = $1 AND user_id = $2 AND role = 'editor'")
                .bind(&id)
                .bind(uid)
                .fetch_optional(&state.db)
                .await 
            {
                is_viewer = false;
            }
        }
        if is_viewer {
            if let Some(pr) = &d.public_role {
                if pr == "editor" {
                    is_viewer = false;
                }
            }
        }
    }

    let mut bcast_map = state.bcast_map.lock().await;
    let bcast = if let Some(bcast) = bcast_map.get(&id) {
        bcast.clone()
    } else {
        let ydoc = Doc::new();
        
        if let Ok(Some(db_doc)) = doc_info {
            if let Some(content) = db_doc.content {
                if let Ok(update) = Update::decode_v1(&content) {
                    ydoc.transact_mut().apply_update(update);
                }
            }
        }
        
        let awareness = Arc::new(RwLock::new(Awareness::new(ydoc)));
        let new_bcast = Arc::new(BroadcastGroup::new(awareness.clone(), 10).await);
        bcast_map.insert(id.clone(), new_bcast.clone());

        let save_db = state.db.clone();
        let save_id = id.clone();
        let save_awareness = awareness.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            loop {
                interval.tick().await;
                let doc = save_awareness.read().await;
                let content = doc.doc().transact().encode_state_as_update_v1(&yrs::StateVector::default());
                let _ = sqlx::query("UPDATE documents SET content = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2")
                    .bind(content)
                    .bind(&save_id)
                    .execute(&save_db)
                    .await;
            }
        });

        new_bcast
    };

    drop(bcast_map);

    ws.on_upgrade(move |socket| async move {
        let (sink, stream) = socket.split();
        let sink = Arc::new(Mutex::new(AxumSink(sink)));
        
        let filtered_stream = ViewerFilterStream {
            inner: stream,
            is_viewer,
        };

        let sub = bcast.subscribe(sink, filtered_stream);
        match sub.completed().await {
            Ok(_) => println!("broadcasting for channel finished successfully"),
            Err(e) => eprintln!("broadcasting for channel finished abruptly: {}", e),
        }
    })
}

pub async fn compile_handler(
    State(state): State<AppState>,
    jar: axum_extra::extract::cookie::SignedCookieJar,
    Json(payload): Json<CompileRequest>,
) -> impl IntoResponse {
    let mut files_map = std::collections::HashMap::new();
    let mut can_save_thumbnail = false;
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());

    if let Some(doc_id) = &payload.document_id {
        if let Ok(doc) = sqlx::query_as::<_, crate::models::Document>("SELECT id, owner_id, folder_id, title, content, thumbnail_svg, public_role, created_at, updated_at FROM documents WHERE id = $1").bind(doc_id).fetch_one(&state.db).await {
            
            // Allow compilation if owner or if it has a public role or if they are a collaborator
            let mut has_access = false;
            if let Some(uid) = &user_id_opt {
                if &doc.owner_id == uid {
                    has_access = true;
                    can_save_thumbnail = true;
                } else if let Ok(Some(_)) = sqlx::query_as::<_, (String,)>("SELECT role FROM collaborators WHERE document_id = $1 AND user_id = $2")
                    .bind(doc_id)
                    .bind(uid)
                    .fetch_optional(&state.db)
                    .await 
                {
                    has_access = true;
                }
            }
            
            if !has_access {
                if let Some(pr) = &doc.public_role {
                    if pr == "viewer" || pr == "editor" {
                        has_access = true;
                    }
                }
            }

            if has_access {
                if let Ok(files) = sqlx::query_as::<_, (String, Vec<u8>)>("SELECT name, data FROM files WHERE owner_id = $1")
                    .bind(doc.owner_id)
                    .fetch_all(&state.db)
                    .await
                {
                    for (name, data) in files {
                        files_map.insert(name, data);
                    }
                }
            }
        }
    }

    let compiler = state.compiler.lock().await;
    match compiler.compile_svg(payload.text, files_map) {
        Ok((svgs, thumbnail)) => {
            if let Some(doc_id) = &payload.document_id {
                if can_save_thumbnail {
                    let _ = sqlx::query("UPDATE documents SET thumbnail_svg = $1 WHERE id = $2")
                        .bind(&thumbnail)
                        .bind(doc_id)
                        .execute(&state.db)
                        .await;
                }
            }
            
            Json(CompileResponse {
                svgs: Some(svgs),
                errors: None,
            })
        }
        Err(diags) => {
            let errors = diags
                .into_iter()
                .map(|d| Diagnostic {
                    message: d.message.to_string(),
                    severity: format!("{:?}", d.severity),
                })
                .collect();
            Json(CompileResponse {
                svgs: None,
                errors: Some(errors),
            })
        }
    }
}

pub async fn export_handler(
    State(state): State<AppState>,
    jar: axum_extra::extract::cookie::SignedCookieJar,
    Path(format): Path<String>,
    Json(payload): Json<CompileRequest>,
) -> impl IntoResponse {
    let mut files_map = std::collections::HashMap::new();
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());

    if let Some(doc_id) = &payload.document_id {
        if let Ok(doc) = sqlx::query_as::<_, crate::models::Document>("SELECT id, owner_id, folder_id, title, content, thumbnail_svg, public_role, created_at, updated_at FROM documents WHERE id = $1").bind(doc_id).fetch_one(&state.db).await {
            
            let mut has_access = false;
            if let Some(uid) = &user_id_opt {
                if &doc.owner_id == uid {
                    has_access = true;
                } else if let Ok(Some(_)) = sqlx::query_as::<_, (String,)>("SELECT role FROM collaborators WHERE document_id = $1 AND user_id = $2")
                    .bind(doc_id)
                    .bind(uid)
                    .fetch_optional(&state.db)
                    .await 
                {
                    has_access = true;
                }
            }
            if !has_access {
                if let Some(pr) = &doc.public_role {
                    if pr == "viewer" || pr == "editor" {
                        has_access = true;
                    }
                }
            }

            if has_access {
                if let Ok(files) = sqlx::query_as::<_, (String, Vec<u8>)>("SELECT name, data FROM files WHERE owner_id = $1")
                    .bind(doc.owner_id)
                    .fetch_all(&state.db)
                    .await
                {
                    for (name, data) in files {
                        files_map.insert(name, data);
                    }
                }
            }
        }
    }

    let compiler = state.compiler.lock().await;

    match format.as_str() {
        "pdf" => match compiler.export_pdf(payload.text, files_map.clone()) {
            Ok(bytes) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/pdf")],
                bytes,
            )
                .into_response(),
            Err(_) => (StatusCode::BAD_REQUEST, "Compilation failed").into_response(),
        },
        "png" => match compiler.export_png(payload.text, files_map.clone()) {
            Ok(bytes) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "image/png")],
                bytes,
            )
                .into_response(),
            Err(_) => (StatusCode::BAD_REQUEST, "Compilation failed").into_response(),
        },
        "svg" => match compiler.compile_svg(payload.text, files_map.clone()) {
            Ok((svgs, _)) => {
                
                
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
    let text = payload.text.clone();
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
                file_ext = "markdown".to_string(); // fallback
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

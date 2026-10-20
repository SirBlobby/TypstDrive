use std::collections::HashMap;
use std::process::Stdio;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::SignedCookieJar;
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

use crate::sharing::access::{self, Resource};
use crate::AppState;

const CONTENT_LENGTH_HEADER: &str = "Content-Length: ";

fn is_safe_name(name: &str) -> bool {
    std::path::Path::new(name)
        .components()
        .all(|part| matches!(part, std::path::Component::Normal(_)))
}

fn write_workspace(files: HashMap<String, Vec<u8>>) -> std::io::Result<tempfile::TempDir> {
    let workspace = tempfile::tempdir()?;
    for (name, data) in files {
        if !is_safe_name(&name) {
            continue;
        }
        let path = workspace.path().join(&name);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, data);
    }
    Ok(workspace)
}

async fn read_message<R: AsyncReadExt + Unpin>(reader: &mut R) -> Option<String> {
    let mut header = Vec::new();
    while !header.ends_with(b"\r\n\r\n") {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte).await.ok()?;
        header.push(byte[0]);
    }

    let content_length = String::from_utf8_lossy(&header)
        .split("\r\n")
        .find_map(|line| line.strip_prefix(CONTENT_LENGTH_HEADER).and_then(|value| value.trim().parse::<usize>().ok()))
        .unwrap_or(0);

    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body).await.ok()?;
    Some(String::from_utf8_lossy(&body).to_string())
}

async fn bridge(state: AppState, socket: WebSocket, document_id: String, user_id: Option<String>, files: HashMap<String, Vec<u8>>) {
    let Ok(workspace) = write_workspace(files) else {
        tracing::warn!("Could not prepare a language server workspace");
        return;
    };

    let spawned = Command::new("tinymist")
        .arg("lsp")
        .arg("--font-path")
        .arg(workspace.path())
        .current_dir(workspace.path())
        .env("RUST_LOG", "warn")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(error) => {
            tracing::warn!("Failed to start tinymist: {error}");
            return;
        }
    };
    let (Some(mut stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
        return;
    };

    let session = access::register_session(&state, Resource::Document(document_id), user_id, None, true).await;
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let init_message = serde_json::json!({
        "type": "init",
        "rootUri": format!("file://{}", workspace.path().display()),
    });
    let _ = ws_sender.send(Message::Text(init_message.to_string().into())).await;

    let mut client_to_server = tokio::spawn(async move {
        while let Some(Ok(Message::Text(message))) = ws_receiver.next().await {
            let header = format!("{CONTENT_LENGTH_HEADER}{}\r\n\r\n", message.len());
            if stdin.write_all(header.as_bytes()).await.is_err() || stdin.write_all(message.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    let mut server_to_client = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout);
        while let Some(message) = read_message(&mut reader).await {
            if message.is_empty() {
                continue;
            }
            if ws_sender.send(Message::Text(message.into())).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = &mut client_to_server => {}
        _ = &mut server_to_client => {}
        _ = child.wait() => {}
        _ = session.revoked => {}
    }

    client_to_server.abort();
    server_to_client.abort();
    let _ = child.kill().await;
    access::unregister_session(&state, &session.resource, session.session_id).await;
}

pub async fn lsp_handler(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Response {
    let user_id = jar.get("session_user_id").map(|cookie| cookie.value().to_string());

    let (document, role) = match access::document_role(&state, &id, &user_id).await {
        Some((document, role)) if role.can_edit() => (document, role),
        Some(_) => return access::forbidden().into_response(),
        None => return (StatusCode::NOT_FOUND, "Document not found").into_response(),
    };

    let files = crate::drive::document_assets::compile_files(&state, &document, role, &user_id).await;
    ws.on_upgrade(move |socket| bridge(state, socket, id, user_id, files))
}

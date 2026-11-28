use std::collections::HashMap;
use std::path::PathBuf;
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

use crate::compile::lsp_guard::LspGuard;
use crate::compile::world::LOCAL_PACKAGE_NAMESPACE;
use crate::sharing::access::{self, Resource};
use crate::AppState;

const CONTENT_LENGTH_HEADER: &str = "Content-Length: ";
const INHERITED_VARIABLES: &[&str] = &["PATH", "HOME", "XDG_CACHE_HOME", "XDG_DATA_HOME", "LANG"];

fn is_safe_name(name: &str) -> bool {
    std::path::Path::new(name)
        .components()
        .all(|part| matches!(part, std::path::Component::Normal(_)))
}

struct LspWorkspace {
    files: HashMap<String, Vec<u8>>,
    packages: HashMap<String, HashMap<String, Vec<u8>>>,
    entry: Option<String>,
    resource: Resource,
}

struct PreparedWorkspace {
    _dir: tempfile::TempDir,
    root: PathBuf,
    package_path: PathBuf,
}

fn write_files(base: &std::path::Path, files: HashMap<String, Vec<u8>>) {
    for (name, data) in files {
        if !is_safe_name(&name) {
            continue;
        }
        let path = base.join(&name);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, data);
    }
}

fn write_workspace(files: HashMap<String, Vec<u8>>, packages: HashMap<String, HashMap<String, Vec<u8>>>) -> std::io::Result<PreparedWorkspace> {
    let dir = tempfile::tempdir()?;
    let root = dir.path().join("project");
    let package_path = dir.path().join("packages");
    std::fs::create_dir_all(&root)?;
    write_files(&root, files);
    for (key, package_files) in packages {
        let Some((name, version)) = key.split_once(':') else {
            continue;
        };
        if !is_safe_name(name) || !is_safe_name(version) || name.contains('/') || version.contains('/') {
            continue;
        }
        write_files(&package_path.join(LOCAL_PACKAGE_NAMESPACE).join(name).join(version), package_files);
    }
    Ok(PreparedWorkspace { _dir: dir, root, package_path })
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

async fn bridge(state: AppState, socket: WebSocket, user_id: Option<String>, spec: LspWorkspace) {
    let Ok(workspace) = write_workspace(spec.files, spec.packages) else {
        tracing::warn!("Could not prepare a language server workspace");
        return;
    };

    let inherited: Vec<(String, String)> = INHERITED_VARIABLES
        .iter()
        .filter_map(|name| std::env::var(name).ok().map(|value| (name.to_string(), value)))
        .collect();
    let spawned = Command::new("tinymist")
        .arg("lsp")
        .arg("--font-path")
        .arg(&workspace.root)
        .current_dir(&workspace.root)
        .env_clear()
        .envs(inherited)
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

    let session = access::register_session(&state, spec.resource, user_id, None, true).await;
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let guard = std::sync::Arc::new(LspGuard::new(&workspace.root, &workspace.package_path, spec.entry.as_deref()));
    let init_message = serde_json::json!({
        "type": "init",
        "rootUri": guard.root_uri(),
    });
    let _ = ws_sender.send(Message::Text(init_message.to_string().into())).await;

    let client_guard = guard.clone();
    let mut client_to_server = tokio::spawn(async move {
        while let Some(Ok(Message::Text(message))) = ws_receiver.next().await {
            for message in client_guard.to_server(&message) {
                let header = format!("{CONTENT_LENGTH_HEADER}{}\r\n\r\n", message.len());
                if stdin.write_all(header.as_bytes()).await.is_err() || stdin.write_all(message.as_bytes()).await.is_err() {
                    return;
                }
            }
        }
    });

    let mut server_to_client = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout);
        while let Some(message) = read_message(&mut reader).await {
            if message.is_empty() || !guard.to_client(&message) {
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
    let spec = LspWorkspace { files, packages: HashMap::new(), entry: None, resource: Resource::Document(id) };
    ws.on_upgrade(move |socket| bridge(state, socket, user_id, spec))
}

pub async fn project_lsp_handler(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Response {
    let user_id = jar.get("session_user_id").map(|cookie| cookie.value().to_string());

    let (project, role) = match access::project_access(&state, &id, &user_id).await {
        Some((project, role)) if role.can_edit() => (project, role),
        Some(_) => return access::forbidden().into_response(),
        None => return (StatusCode::NOT_FOUND, "Project not found").into_response(),
    };
    if !project.entrypoint.ends_with(".typ") {
        return (StatusCode::BAD_REQUEST, "Project entrypoint is not a Typst file").into_response();
    }

    let reader = crate::drive::projects::ProjectReader::for_role(role, &user_id);
    let input = crate::drive::projects::assemble_project(&state, &project, HashMap::new(), reader).await;
    let spec = LspWorkspace {
        files: input.files,
        packages: input.packages,
        entry: Some(input.entrypoint),
        resource: Resource::Project(id),
    };
    ws.on_upgrade(move |socket| bridge(state, socket, user_id, spec))
}

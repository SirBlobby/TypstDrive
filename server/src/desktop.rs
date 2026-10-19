use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::access::AccessRole;
use crate::{
    devices::{notify_devices, DeviceEvent},
    models::{Project, User},
    projects::{decode_text_blob, encode_text_blob},
    AppState,
};

const TEXT_EXTENSIONS: [&str; 10] = [
    ".typ", ".toml", ".bib", ".csl", ".yml", ".yaml", ".json", ".md", ".txt", ".csv",
];

pub const MIN_DESKTOP_VERSION: &str = "1.0.0";

fn parse_version(version: &str) -> (u32, u32, u32) {
    let mut parts = version.trim().split('.').map(|part| part.parse::<u32>().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

fn version_at_least(actual: &str, required: &str) -> bool {
    parse_version(actual) >= parse_version(required)
}

#[derive(Serialize)]
pub struct ServerVersionInfo {
    pub server_version: String,
    pub min_desktop_version: String,
}

pub async fn version_info() -> Json<ServerVersionInfo> {
    Json(ServerVersionInfo {
        server_version: env!("CARGO_PKG_VERSION").to_string(),
        min_desktop_version: MIN_DESKTOP_VERSION.to_string(),
    })
}

fn is_text_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    TEXT_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

fn content_hash(bytes: &[u8]) -> String {
    crate::hashing::sha256_hex(bytes)
}

pub(crate) fn hash_token(token: &str) -> String {
    crate::hashing::sha256_hex(token.as_bytes())
}

fn generate_token() -> String {
    format!(
        "tdd_{}{}",
        Uuid::new_v4().to_string().replace("-", ""),
        Uuid::new_v4().to_string().replace("-", "")
    )
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.starts_with("Bearer "))
        .map(|value| value[7..].to_string())
}

pub async fn authenticate(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<String, (StatusCode, String)> {
    let token = bearer_token(headers).ok_or((
        StatusCode::UNAUTHORIZED,
        "Missing Authorization header. Use: Authorization: Bearer <device-token>".to_string(),
    ))?;

    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT id, user_id FROM device_tokens WHERE token_hash = $1",
    )
    .bind(hash_token(&token))
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (token_id, user_id) =
        row.ok_or((StatusCode::UNAUTHORIZED, "Invalid device token".to_string()))?;

    let _ = sqlx::query("UPDATE device_tokens SET last_used_at = $1 WHERE id = $2")
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(&token_id)
        .execute(&state.db)
        .await;

    Ok(user_id)
}

pub(crate) async fn user_id_for_token(state: &AppState, token: &str) -> Option<String> {
    device_token_for(state, token).await.map(|(_, user_id)| user_id)
}

pub(crate) async fn device_token_for(state: &AppState, token: &str) -> Option<(String, String)> {
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT id, user_id FROM device_tokens WHERE token_hash = $1",
    )
    .bind(hash_token(token))
    .fetch_optional(&state.db)
    .await
    .ok()??;

    let (token_id, user_id) = row;

    let _ = sqlx::query("UPDATE device_tokens SET last_used_at = $1 WHERE id = $2")
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(&token_id)
        .execute(&state.db)
        .await;

    Some((token_id, user_id))
}

type DesktopError = (StatusCode, String);

async fn project_with_role(
    state: &AppState,
    project_id: &str,
    user_id: &str,
) -> Result<(Project, AccessRole), DesktopError> {
    crate::access::project_access(state, project_id, &Some(user_id.to_string()))
        .await
        .ok_or((StatusCode::NOT_FOUND, "Project not found".to_string()))
}

async fn readable_project(state: &AppState, project_id: &str, user_id: &str) -> Result<Project, DesktopError> {
    Ok(project_with_role(state, project_id, user_id).await?.0)
}

async fn writable_project(state: &AppState, project_id: &str, user_id: &str) -> Result<Project, DesktopError> {
    let (project, role) = project_with_role(state, project_id, user_id).await?;
    if !role.can_edit() {
        return Err((StatusCode::FORBIDDEN, "Read-only access".to_string()));
    }
    Ok(project)
}

async fn document_owner(state: &AppState, document_id: &str) -> Option<String> {
    sqlx::query_as::<_, (String,)>("SELECT owner_id FROM documents WHERE id = $1")
        .bind(document_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .map(|(owner_id,)| owner_id)
}

#[derive(Deserialize)]
pub struct DeviceLoginRequest {
    pub email: String,
    pub password: String,
    pub device_name: Option<String>,
    pub client_version: Option<String>,
}

#[derive(Serialize)]
pub struct DeviceLoginResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub email: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<DeviceLoginRequest>,
) -> Result<Json<DeviceLoginResponse>, (StatusCode, String)> {
    if let Some(client_version) = &payload.client_version {
        if !version_at_least(client_version, MIN_DESKTOP_VERSION) {
            return Err((
                StatusCode::UPGRADE_REQUIRED,
                format!(
                    "This server requires typst-desktop v{} or newer (you have v{}). Please update the app.",
                    MIN_DESKTOP_VERSION, client_version
                ),
            ));
        }
    }

    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, is_admin FROM users WHERE email = $1",
    )
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((
        StatusCode::UNAUTHORIZED,
        "Invalid email or password".to_string(),
    ))?;

    if !crate::passwords::verify_password(&payload.password, &user.password_hash) {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid email or password".to_string(),
        ));
    }

    let token = generate_token();
    let device_name = payload
        .device_name
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| "Typst Desktop".to_string());

    sqlx::query(
        "INSERT INTO device_tokens (id, user_id, name, token_hash, created_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&user.id)
    .bind(&device_name)
    .bind(hash_token(&token))
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(DeviceLoginResponse {
        token,
        user_id: user.id,
        username: user.username,
        email: user.email,
    }))
}

#[derive(Serialize)]
pub struct DeviceUser {
    pub user_id: String,
    pub username: String,
    pub email: String,
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DeviceUser>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, is_admin FROM users WHERE id = $1",
    )
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    Ok(Json(DeviceUser {
        user_id: user.id,
        username: user.username,
        email: user.email,
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, String)> {
    let token = bearer_token(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        "Missing Authorization header".to_string(),
    ))?;

    let token_id = sqlx::query_as::<_, (String,)>("SELECT id FROM device_tokens WHERE token_hash = $1")
        .bind(hash_token(&token))
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("DELETE FROM device_tokens WHERE token_hash = $1")
        .bind(hash_token(&token))
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some((token_id,)) = token_id {
        crate::access::revoke_device_sessions(&state, &token_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub entrypoint: String,
    pub folder_id: Option<String>,
    pub role: String,
    pub updated_at: String,
}

pub async fn list_projects(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ProjectSummary>>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let owned = sqlx::query_as::<_, (String, String, String, Option<String>, String)>(
        "SELECT id, name, entrypoint, folder_id, updated_at FROM projects WHERE owner_id = $1 ORDER BY updated_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let shared = sqlx::query_as::<_, (String, String, String, Option<String>, String, String)>(
        "SELECT p.id, p.name, p.entrypoint, p.folder_id, p.updated_at, c.role FROM projects p \
         INNER JOIN project_collaborators c ON c.project_id = p.id AND c.user_id = $1 \
         ORDER BY p.updated_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut projects: Vec<ProjectSummary> = owned
        .into_iter()
        .map(|(id, name, entrypoint, folder_id, updated_at)| ProjectSummary {
            id,
            name,
            entrypoint,
            folder_id,
            role: "owner".to_string(),
            updated_at,
        })
        .collect();

    projects.extend(
        shared
            .into_iter()
            .map(|(id, name, entrypoint, folder_id, updated_at, role)| ProjectSummary {
                id,
                name,
                entrypoint,
                folder_id,
                role,
                updated_at,
            }),
    );

    Ok(Json(projects))
}

#[derive(Deserialize)]
pub struct CreateProjectBody {
    pub name: String,
    pub entrypoint: Option<String>,
    pub folder_id: Option<String>,
}

pub async fn create_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateProjectBody>,
) -> Result<Json<ProjectSummary>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }

    if let Some(folder_id) = &payload.folder_id {
        owned_folder(&state, folder_id, &user_id).await?;
    }

    let project_id = Uuid::new_v4().to_string();
    let entrypoint = payload
        .entrypoint
        .unwrap_or_else(|| "main.typ".to_string());

    let project = sqlx::query_as::<_, Project>(
        "INSERT INTO projects (id, owner_id, folder_id, name, entrypoint) VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at",
    )
    .bind(&project_id)
    .bind(&user_id)
    .bind(&payload.folder_id)
    .bind(payload.name.trim())
    .bind(&entrypoint)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(ProjectSummary {
        id: project.id,
        name: project.name,
        entrypoint: project.entrypoint,
        folder_id: project.folder_id,
        role: "owner".to_string(),
        updated_at: project.updated_at,
    }))
}

pub async fn delete_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let (_, role) = project_with_role(&state, &project_id, &user_id).await?;
    if role != AccessRole::Owner {
        return Err((StatusCode::FORBIDDEN, "Only the owner can delete this project".to_string()));
    }

    sqlx::query("DELETE FROM projects WHERE id = $1 AND owner_id = $2")
        .bind(&project_id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::access::revalidate_resource(&state, crate::access::Resource::Project(project_id.clone())).await;
    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct ManifestEntry {
    pub id: String,
    pub path: String,
    pub kind: String,
    pub hash: String,
    pub size: usize,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct ProjectManifest {
    pub project_id: String,
    pub name: String,
    pub entrypoint: String,
    pub updated_at: String,
    pub files: Vec<ManifestEntry>,
}

async fn plain_contents(
    state: &AppState,
    project_id: &str,
) -> Result<Vec<(String, String, String, Vec<u8>, String)>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, (String, String, String, Option<Vec<u8>>, Option<String>)>(
        "SELECT id, path, kind, content, updated_at FROM project_files WHERE project_id = $1 ORDER BY path ASC",
    )
    .bind(project_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut contents = Vec::with_capacity(rows.len());
    for (id, path, kind, content, updated_at) in rows {
        let plain = plain_file_content(state, project_id, &id, &kind, content).await;
        contents.push((id, path, kind, plain, updated_at.unwrap_or_default()));
    }
    Ok(contents)
}

async fn plain_file_content(
    state: &AppState,
    project_id: &str,
    file_id: &str,
    kind: &str,
    stored: Option<Vec<u8>>,
) -> Vec<u8> {
    let raw = stored.unwrap_or_default();
    if kind == "binary" {
        return raw;
    }
    let room = crate::live_documents::project_file_room(project_id, file_id);
    match crate::live_documents::live_text(state, &room).await {
        Some(live) => live.into_bytes(),
        None => decode_text_blob(&raw).into_bytes(),
    }
}

pub async fn get_manifest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
) -> Result<Json<ProjectManifest>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    let project = readable_project(&state, &project_id, &user_id).await?;

    let files = plain_contents(&state, &project_id)
        .await?
        .into_iter()
        .map(|(id, path, kind, plain, updated_at)| ManifestEntry {
            id,
            path,
            kind,
            hash: content_hash(&plain),
            size: plain.len(),
            updated_at,
        })
        .collect();

    Ok(Json(ProjectManifest {
        project_id: project.id,
        name: project.name,
        entrypoint: project.entrypoint,
        updated_at: project.updated_at,
        files,
    }))
}

#[derive(Deserialize)]
pub struct PathQuery {
    pub path: String,
}

#[derive(Serialize)]
pub struct FileContent {
    pub path: String,
    pub kind: String,
    pub hash: String,
    pub encoding: String,
    pub content: String,
}

fn encode_for_transport(kind: &str, plain: Vec<u8>) -> (String, String) {
    if kind == "binary" {
        ("base64".to_string(), BASE64.encode(&plain))
    } else {
        (
            "utf8".to_string(),
            String::from_utf8_lossy(&plain).to_string(),
        )
    }
}

pub async fn pull_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    Query(query): Query<PathQuery>,
) -> Result<Json<FileContent>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    readable_project(&state, &project_id, &user_id).await?;

    let row = sqlx::query_as::<_, (String, String, Option<Vec<u8>>)>(
        "SELECT id, kind, content FROM project_files WHERE project_id = $1 AND path = $2",
    )
    .bind(&project_id)
    .bind(&query.path)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    let (file_id, kind, content) = row;
    let plain = plain_file_content(&state, &project_id, &file_id, &kind, content).await;

    let hash = content_hash(&plain);
    let (encoding, content) = encode_for_transport(&kind, plain);

    Ok(Json(FileContent {
        path: query.path,
        kind,
        hash,
        encoding,
        content,
    }))
}

#[derive(Deserialize)]
pub struct PushFileRequest {
    pub path: String,
    pub content: String,
    pub encoding: Option<String>,
    pub base_hash: Option<String>,
}

#[derive(Serialize)]
pub struct PushFileResponse {
    pub path: String,
    pub hash: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct ConflictResponse {
    pub conflict: bool,
    pub path: String,
    pub server_hash: String,
    pub base_hash: Option<String>,
    pub encoding: String,
    pub server_content: String,
}

pub enum PushOutcome {
    Applied(Json<PushFileResponse>),
    Conflict(Json<ConflictResponse>),
}

impl axum::response::IntoResponse for PushOutcome {
    fn into_response(self) -> axum::response::Response {
        match self {
            PushOutcome::Applied(body) => (StatusCode::OK, body).into_response(),
            PushOutcome::Conflict(body) => (StatusCode::CONFLICT, body).into_response(),
        }
    }
}

pub async fn push_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    Json(payload): Json<PushFileRequest>,
) -> Result<PushOutcome, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    let project = writable_project(&state, &project_id, &user_id).await?;

    let incoming = match payload.encoding.as_deref() {
        Some("base64") => BASE64
            .decode(payload.content.as_bytes())
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid base64: {}", e)))?,
        _ => payload.content.clone().into_bytes(),
    };

    let existing = sqlx::query_as::<_, (String, String, Option<Vec<u8>>)>(
        "SELECT id, kind, content FROM project_files WHERE project_id = $1 AND path = $2",
    )
    .bind(&project_id)
    .bind(&payload.path)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some((existing_id, existing_kind, existing_content)) = &existing {
        let plain = plain_file_content(&state, &project_id, existing_id, existing_kind, existing_content.clone()).await;
        let server_hash = content_hash(&plain);

        let safe = match &payload.base_hash {
            Some(base) => base == &server_hash,
            None => false,
        };

        if !safe && server_hash != content_hash(&incoming) {
            let (encoding, server_content) = encode_for_transport(existing_kind, plain);
            return Ok(PushOutcome::Conflict(Json(ConflictResponse {
                conflict: true,
                path: payload.path,
                server_hash,
                base_hash: payload.base_hash,
                encoding,
                server_content,
            })));
        }
    }

    let kind = if payload.encoding.as_deref() == Some("base64") && !is_text_path(&payload.path) {
        "binary"
    } else {
        "text"
    };

    let stored = if kind == "binary" {
        incoming.clone()
    } else {
        encode_text_blob(&String::from_utf8_lossy(&incoming))
    };

    let mime_type = if kind == "binary" {
        "application/octet-stream"
    } else {
        "text/plain"
    };

    let existing_size = existing
        .as_ref()
        .and_then(|(_, _, content)| content.as_ref())
        .map(|content| content.len() as i64)
        .unwrap_or(0);
    crate::storage::ensure_room(&state, &project.owner_id, stored.len() as i64 - existing_size).await?;

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    sqlx::query(
        "INSERT INTO project_files (id, project_id, path, kind, content, mime_type, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         ON CONFLICT (project_id, path) DO UPDATE SET content = excluded.content, \
         kind = excluded.kind, mime_type = excluded.mime_type, updated_at = excluded.updated_at",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&project_id)
    .bind(&payload.path)
    .bind(kind)
    .bind(&stored)
    .bind(mime_type)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = sqlx::query("UPDATE projects SET updated_at = $1 WHERE id = $2")
        .bind(&now)
        .bind(&project_id)
        .execute(&state.db)
        .await;

    if kind == "text" {
        let file_id = sqlx::query_as::<_, (String,)>("SELECT id FROM project_files WHERE project_id = $1 AND path = $2")
            .bind(&project_id)
            .bind(&payload.path)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();
        if let Some((file_id,)) = file_id {
            let room = crate::live_documents::project_file_room(&project_id, &file_id);
            crate::live_documents::replace_text(&state, &room, &String::from_utf8_lossy(&incoming)).await;
        }
    }

    notify_devices(&state, &project.owner_id, DeviceEvent::project(&project_id)).await;

    Ok(PushOutcome::Applied(Json(PushFileResponse {
        path: payload.path,
        hash: content_hash(&incoming),
        updated_at: now,
    })))
}

pub async fn delete_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    Query(query): Query<PathQuery>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    writable_project(&state, &project_id, &user_id).await?;

    let result = sqlx::query("DELETE FROM project_files WHERE project_id = $1 AND path = $2")
        .bind(&project_id)
        .bind(&query.path)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    notify_devices(&state, &user_id, DeviceEvent::project(&project_id)).await;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct BundleFile {
    pub path: String,
    pub kind: String,
    pub hash: String,
    pub encoding: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ProjectBundle {
    pub project_id: String,
    pub name: String,
    pub entrypoint: String,
    pub files: Vec<BundleFile>,
}

pub async fn pull_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
) -> Result<Json<ProjectBundle>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    let project = readable_project(&state, &project_id, &user_id).await?;

    let files = plain_contents(&state, &project_id)
        .await?
        .into_iter()
        .map(|(_, path, kind, plain, _)| {
            let hash = content_hash(&plain);
            let (encoding, content) = encode_for_transport(&kind, plain);
            BundleFile {
                path,
                kind,
                hash,
                encoding,
                content,
            }
        })
        .collect();

    Ok(Json(ProjectBundle {
        project_id: project.id,
        name: project.name,
        entrypoint: project.entrypoint,
        files,
    }))
}

#[derive(Serialize)]
pub struct CloudFolder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

pub async fn list_folders(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<CloudFolder>>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT id, name, parent_id FROM folders WHERE owner_id = $1 ORDER BY name ASC",
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, name, parent_id)| CloudFolder {
                id,
                name,
                parent_id,
            })
            .collect(),
    ))
}

async fn owned_folder(
    state: &AppState,
    folder_id: &str,
    user_id: &str,
) -> Result<(), (StatusCode, String)> {
    let row = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM folders WHERE id = $1 AND owner_id = $2",
    )
    .bind(folder_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if row.0 == 0 {
        return Err((StatusCode::NOT_FOUND, "Folder not found".to_string()));
    }

    Ok(())
}

#[derive(Deserialize)]
pub struct CreateFolderBody {
    pub name: String,
    pub parent_id: Option<String>,
}

pub async fn create_folder(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateFolderBody>,
) -> Result<Json<CloudFolder>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }

    if let Some(parent_id) = &payload.parent_id {
        owned_folder(&state, parent_id, &user_id).await?;
    }

    let folder_id = Uuid::new_v4().to_string();
    let name = payload.name.trim().to_string();

    sqlx::query("INSERT INTO folders (id, owner_id, parent_id, name) VALUES ($1, $2, $3, $4)")
        .bind(&folder_id)
        .bind(&user_id)
        .bind(&payload.parent_id)
        .bind(&name)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(CloudFolder {
        id: folder_id,
        name,
        parent_id: payload.parent_id,
    }))
}

#[derive(Deserialize)]
pub struct RenameFolderBody {
    pub name: String,
}

pub async fn rename_folder(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<RenameFolderBody>,
) -> Result<Json<CloudFolder>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }

    let row = sqlx::query_as::<_, (String, Option<String>)>(
        "UPDATE folders SET name = $1 WHERE id = $2 AND owner_id = $3 RETURNING name, parent_id",
    )
    .bind(payload.name.trim())
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Folder not found".to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(CloudFolder {
        id,
        name: row.0,
        parent_id: row.1,
    }))
}

async fn creates_cycle(
    state: &AppState,
    folder_id: &str,
    new_parent_id: &str,
    user_id: &str,
) -> Result<bool, (StatusCode, String)> {
    let mut current = new_parent_id.to_string();

    for _ in 0..1000 {
        if current == folder_id {
            return Ok(true);
        }

        let parent = sqlx::query_as::<_, (Option<String>,)>(
            "SELECT parent_id FROM folders WHERE id = $1 AND owner_id = $2",
        )
        .bind(&current)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .and_then(|row| row.0);

        match parent {
            Some(next) => current = next,
            None => return Ok(false),
        }
    }

    Ok(true)
}

#[derive(Deserialize)]
pub struct MoveFolderBody {
    pub parent_id: Option<String>,
}

pub async fn move_folder(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<MoveFolderBody>,
) -> Result<Json<CloudFolder>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    owned_folder(&state, &id, &user_id).await?;

    if let Some(parent_id) = &payload.parent_id {
        if parent_id == &id {
            return Err((
                StatusCode::BAD_REQUEST,
                "A folder cannot be moved into itself".to_string(),
            ));
        }

        owned_folder(&state, parent_id, &user_id).await?;

        if creates_cycle(&state, &id, parent_id, &user_id).await? {
            return Err((
                StatusCode::BAD_REQUEST,
                "Cannot move a folder into one of its own subfolders".to_string(),
            ));
        }
    }

    let row = sqlx::query_as::<_, (String, Option<String>)>(
        "UPDATE folders SET parent_id = $1 WHERE id = $2 AND owner_id = $3 RETURNING name, parent_id",
    )
    .bind(&payload.parent_id)
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Folder not found".to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(CloudFolder {
        id,
        name: row.0,
        parent_id: row.1,
    }))
}

async fn folder_is_empty(
    state: &AppState,
    folder_id: &str,
    user_id: &str,
) -> Result<bool, (StatusCode, String)> {
    let row = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT \
           (SELECT COUNT(*) FROM folders WHERE parent_id = $1 AND owner_id = $2), \
           (SELECT COUNT(*) FROM projects WHERE folder_id = $3 AND owner_id = $4), \
           (SELECT COUNT(*) FROM documents WHERE folder_id = $5 AND owner_id = $6)",
    )
    .bind(folder_id)
    .bind(user_id)
    .bind(folder_id)
    .bind(user_id)
    .bind(folder_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(row.0 == 0 && row.1 == 0 && row.2 == 0)
}

pub async fn delete_folder(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    owned_folder(&state, &id, &user_id).await?;

    if !folder_is_empty(&state, &id, &user_id).await? {
        return Err((
            StatusCode::BAD_REQUEST,
            "Move or delete everything inside this folder first".to_string(),
        ));
    }

    let result = sqlx::query("DELETE FROM folders WHERE id = $1 AND owner_id = $2")
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Folder not found or unauthorized".to_string()));
    }

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct MoveProjectBody {
    pub folder_id: Option<String>,
}

pub async fn move_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    Json(payload): Json<MoveProjectBody>,
) -> Result<Json<ProjectSummary>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    if let Some(folder_id) = &payload.folder_id {
        owned_folder(&state, folder_id, &user_id).await?;
    }

    let row = sqlx::query_as::<_, (String, String, String, Option<String>, String)>(
        "UPDATE projects SET folder_id = $1 WHERE id = $2 AND owner_id = $3 \
         RETURNING id, name, entrypoint, folder_id, updated_at",
    )
    .bind(&payload.folder_id)
    .bind(&project_id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Project not found".to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(ProjectSummary {
        id: row.0,
        name: row.1,
        entrypoint: row.2,
        folder_id: row.3,
        role: "owner".to_string(),
        updated_at: row.4,
    }))
}

#[derive(Serialize)]
pub struct CloudDocument {
    pub id: String,
    pub title: String,
    pub folder_id: Option<String>,
    pub role: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct FolderQuery {
    pub folder_id: Option<String>,
}

pub async fn list_documents(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<FolderQuery>,
) -> Result<Json<Vec<CloudDocument>>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let rows = match &query.folder_id {
        Some(folder_id) => sqlx::query_as::<_, (String, String, Option<String>, String)>(
            "SELECT id, title, folder_id, updated_at FROM documents \
             WHERE owner_id = $1 AND folder_id = $2 ORDER BY updated_at DESC",
        )
        .bind(&user_id)
        .bind(folder_id)
        .fetch_all(&state.db)
        .await,
        None => sqlx::query_as::<_, (String, String, Option<String>, String)>(
            "SELECT id, title, folder_id, updated_at FROM documents \
             WHERE owner_id = $1 AND folder_id IS NULL ORDER BY updated_at DESC",
        )
        .bind(&user_id)
        .fetch_all(&state.db)
        .await,
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, title, folder_id, updated_at)| CloudDocument {
                id,
                title,
                folder_id,
                role: "owner".to_string(),
                updated_at,
            })
            .collect(),
    ))
}

#[derive(Serialize)]
pub struct SharedItems {
    pub documents: Vec<CloudDocument>,
    pub projects: Vec<ProjectSummary>,
}

pub async fn list_shared(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<SharedItems>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let documents = sqlx::query_as::<_, (String, String, Option<String>, String, String)>(
        "SELECT d.id, d.title, d.folder_id, d.updated_at, c.role FROM documents d \
         INNER JOIN collaborators c ON c.document_id = d.id AND c.user_id = $1 \
         ORDER BY d.updated_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let projects = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT p.id, p.name, p.entrypoint, p.updated_at, c.role FROM projects p \
         INNER JOIN project_collaborators c ON c.project_id = p.id AND c.user_id = $1 \
         ORDER BY p.updated_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(SharedItems {
        documents: documents
            .into_iter()
            .map(|(id, title, folder_id, updated_at, role)| CloudDocument {
                id,
                title,
                folder_id,
                role,
                updated_at,
            })
            .collect(),
        projects: projects
            .into_iter()
            .map(|(id, name, entrypoint, updated_at, role)| ProjectSummary {
                id,
                name,
                entrypoint,
                folder_id: None,
                role,
                updated_at,
            })
            .collect(),
    }))
}

async fn document_role(
    state: &AppState,
    document_id: &str,
    user_id: &str,
) -> Result<(String, String, String), (StatusCode, String)> {
    let row = sqlx::query_as::<_, (String, String, Option<Vec<u8>>, Option<String>)>(
        "SELECT owner_id, title, content, public_role FROM documents WHERE id = $1",
    )
    .bind(document_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;

    let (owner_id, title, content, public_role) = row;

    let mut role = if owner_id == user_id {
        "owner".to_string()
    } else {
        "none".to_string()
    };

    if role == "none" {
        if let Ok(Some((collaborator_role,))) = sqlx::query_as::<_, (String,)>(
            "SELECT role FROM collaborators WHERE document_id = $1 AND user_id = $2",
        )
        .bind(document_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        {
            role = collaborator_role;
        }
    }

    if role == "none" {
        if let Some(public) = public_role {
            if public == "viewer" || public == "editor" {
                role = public;
            }
        }
    }

    if role == "none" {
        return Err((StatusCode::FORBIDDEN, "No access to this document".to_string()));
    }

    let text = decode_text_blob(&content.unwrap_or_default());
    Ok((role, title, text))
}

#[derive(Serialize)]
pub struct DocumentContent {
    pub id: String,
    pub title: String,
    pub role: String,
    pub hash: String,
    pub content: String,
}

pub async fn pull_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<DocumentContent>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    let (role, title, stored_content) = document_role(&state, &id, &user_id).await?;
    let content = crate::live_documents::live_text(&state, &crate::live_documents::document_room(&id))
        .await
        .unwrap_or(stored_content);

    Ok(Json(DocumentContent {
        id,
        title,
        role,
        hash: content_hash(content.as_bytes()),
        content,
    }))
}

#[derive(Deserialize)]
pub struct PushDocumentRequest {
    pub content: String,
    pub base_hash: Option<String>,
}

pub async fn push_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<PushDocumentRequest>,
) -> Result<PushOutcome, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    let (role, _, stored_content) = document_role(&state, &id, &user_id).await?;

    if role != "owner" && role != "editor" {
        return Err((StatusCode::FORBIDDEN, "Read-only access".to_string()));
    }

    let room = crate::live_documents::document_room(&id);
    let current = crate::live_documents::live_text(&state, &room).await.unwrap_or(stored_content);

    let server_hash = content_hash(current.as_bytes());
    let incoming_hash = content_hash(payload.content.as_bytes());

    let safe = match &payload.base_hash {
        Some(base) => base == &server_hash,
        None => false,
    };

    if !safe && server_hash != incoming_hash {
        return Ok(PushOutcome::Conflict(Json(ConflictResponse {
            conflict: true,
            path: id,
            server_hash,
            base_hash: payload.base_hash,
            encoding: "utf8".to_string(),
            server_content: current,
        })));
    }

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    sqlx::query("UPDATE documents SET content = $1, updated_at = $2 WHERE id = $3")
        .bind(encode_text_blob(&payload.content))
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::live_documents::replace_text(&state, &room, &payload.content).await;
    let owner_id = document_owner(&state, &id).await.unwrap_or(user_id);
    notify_devices(&state, &owner_id, DeviceEvent::document(&id)).await;

    Ok(PushOutcome::Applied(Json(PushFileResponse {
        path: id,
        hash: incoming_hash,
        updated_at: now,
    })))
}

#[derive(Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub content: String,
    pub folder_id: Option<String>,
}

pub async fn create_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateDocumentRequest>,
) -> Result<Json<DocumentContent>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    crate::storage::ensure_room(&state, &user_id, payload.content.len() as i64).await?;
    let document_id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO documents (id, owner_id, folder_id, title, content) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&document_id)
    .bind(&user_id)
    .bind(&payload.folder_id)
    .bind(&payload.title)
    .bind(encode_text_blob(&payload.content))
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(DocumentContent {
        id: document_id,
        title: payload.title,
        role: "owner".to_string(),
        hash: content_hash(payload.content.as_bytes()),
        content: payload.content,
    }))
}

#[derive(Deserialize)]
pub struct MoveDocumentBody {
    pub folder_id: Option<String>,
}

pub async fn move_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<MoveDocumentBody>,
) -> Result<Json<CloudDocument>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    if let Some(folder_id) = &payload.folder_id {
        owned_folder(&state, folder_id, &user_id).await?;
    }

    let row = sqlx::query_as::<_, (String, String, Option<String>, String)>(
        "UPDATE documents SET folder_id = $1 WHERE id = $2 AND owner_id = $3 \
         RETURNING id, title, folder_id, updated_at",
    )
    .bind(&payload.folder_id)
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(CloudDocument {
        id: row.0,
        title: row.1,
        folder_id: row.2,
        role: "owner".to_string(),
        updated_at: row.3,
    }))
}

pub async fn delete_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let result = sqlx::query("DELETE FROM documents WHERE id = $1 AND owner_id = $2")
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Document not found or unauthorized".to_string()));
    }

    crate::access::revalidate_resource(&state, crate::access::Resource::Document(id.clone())).await;
    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct CloudFile {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub folder_id: Option<String>,
    pub created_at: String,
}

pub async fn list_account_files(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<FolderQuery>,
) -> Result<Json<Vec<CloudFile>>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let rows = match &query.folder_id {
        Some(folder_id) => sqlx::query_as::<_, (String, String, String, Option<String>, String)>(
            "SELECT id, name, mime_type, folder_id, created_at FROM files \
             WHERE owner_id = $1 AND folder_id = $2 ORDER BY name ASC",
        )
        .bind(&user_id)
        .bind(folder_id)
        .fetch_all(&state.db)
        .await,
        None => sqlx::query_as::<_, (String, String, String, Option<String>, String)>(
            "SELECT id, name, mime_type, folder_id, created_at FROM files \
             WHERE owner_id = $1 AND folder_id IS NULL ORDER BY name ASC",
        )
        .bind(&user_id)
        .fetch_all(&state.db)
        .await,
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, name, mime_type, folder_id, created_at)| CloudFile {
                id,
                name,
                mime_type,
                folder_id,
                created_at,
            })
            .collect(),
    ))
}

#[derive(Deserialize)]
pub struct UploadFileBody {
    pub name: String,
    pub mime_type: String,
    pub encoding: String,
    pub content: String,
    pub folder_id: Option<String>,
}

pub async fn upload_account_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UploadFileBody>,
) -> Result<Json<CloudFile>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }

    if let Some(folder_id) = &payload.folder_id {
        owned_folder(&state, folder_id, &user_id).await?;
    }

    let data = match payload.encoding.as_str() {
        "base64" => BASE64
            .decode(payload.content.as_bytes())
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid base64: {}", e)))?,
        _ => payload.content.into_bytes(),
    };

    crate::storage::ensure_room(&state, &user_id, data.len() as i64).await?;

    let file_id = Uuid::new_v4().to_string();
    let name = payload.name.trim().to_string();

    let row = sqlx::query_as::<_, (String,)>(
        "INSERT INTO files (id, owner_id, folder_id, name, mime_type, data) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING created_at",
    )
    .bind(&file_id)
    .bind(&user_id)
    .bind(&payload.folder_id)
    .bind(&name)
    .bind(&payload.mime_type)
    .bind(&data)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(CloudFile {
        id: file_id,
        name,
        mime_type: payload.mime_type,
        folder_id: payload.folder_id,
        created_at: row.0,
    }))
}

#[derive(Deserialize)]
pub struct RenameFileBody {
    pub name: String,
}

pub async fn rename_account_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<RenameFileBody>,
) -> Result<Json<CloudFile>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }

    let row = sqlx::query_as::<_, (String, String, Option<String>, String)>(
        "UPDATE files SET name = $1 WHERE id = $2 AND owner_id = $3 \
         RETURNING name, mime_type, folder_id, created_at",
    )
    .bind(payload.name.trim())
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(CloudFile {
        id,
        name: row.0,
        mime_type: row.1,
        folder_id: row.2,
        created_at: row.3,
    }))
}

#[derive(Deserialize)]
pub struct MoveFileBody {
    pub folder_id: Option<String>,
}

pub async fn move_account_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<MoveFileBody>,
) -> Result<Json<CloudFile>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    if let Some(folder_id) = &payload.folder_id {
        owned_folder(&state, folder_id, &user_id).await?;
    }

    let row = sqlx::query_as::<_, (String, String, Option<String>, String)>(
        "UPDATE files SET folder_id = $1 WHERE id = $2 AND owner_id = $3 \
         RETURNING name, mime_type, folder_id, created_at",
    )
    .bind(&payload.folder_id)
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(CloudFile {
        id,
        name: row.0,
        mime_type: row.1,
        folder_id: row.2,
        created_at: row.3,
    }))
}

#[derive(Serialize)]
pub struct CloudFileContent {
    pub name: String,
    pub mime_type: String,
    pub encoding: String,
    pub content: String,
}

pub async fn pull_account_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<CloudFileContent>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let row = sqlx::query_as::<_, (String, String, Vec<u8>)>(
        "SELECT name, mime_type, data FROM files WHERE id = $1 AND owner_id = $2",
    )
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    let (name, mime_type, data) = row;

    Ok(Json(CloudFileContent {
        name,
        mime_type,
        encoding: "base64".to_string(),
        content: BASE64.encode(&data),
    }))
}

pub async fn delete_account_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let result = sqlx::query("DELETE FROM files WHERE id = $1 AND owner_id = $2")
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "File not found or unauthorized".to_string()));
    }

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(StatusCode::NO_CONTENT)
}

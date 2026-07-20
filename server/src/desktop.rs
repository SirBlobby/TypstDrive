use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};

use crate::{
    models::{Project, User},
    projects::{decode_text_blob, encode_text_blob},
    AppState,
};

const TEXT_EXTENSIONS: [&str; 10] = [
    ".typ", ".toml", ".bib", ".csl", ".yml", ".yaml", ".json", ".md", ".txt", ".csv",
];

fn is_text_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    TEXT_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

fn content_hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hash_token(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
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
        "SELECT id, user_id FROM device_tokens WHERE token_hash = ?",
    )
    .bind(hash_token(&token))
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (token_id, user_id) =
        row.ok_or((StatusCode::UNAUTHORIZED, "Invalid device token".to_string()))?;

    let _ = sqlx::query("UPDATE device_tokens SET last_used_at = ? WHERE id = ?")
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(&token_id)
        .execute(&state.db)
        .await;

    Ok(user_id)
}

async fn owned_project(
    state: &AppState,
    project_id: &str,
    user_id: &str,
) -> Result<Project, (StatusCode, String)> {
    let project = sqlx::query_as::<_, Project>(
        "SELECT p.id, p.owner_id, p.folder_id, p.name, p.entrypoint, p.thumbnail_svg, \
         p.public_role, p.created_at, p.updated_at FROM projects p \
         WHERE p.id = ? AND (p.owner_id = ? OR EXISTS ( \
           SELECT 1 FROM project_collaborators c \
           WHERE c.project_id = p.id AND c.user_id = ? AND c.role = 'editor'))",
    )
    .bind(project_id)
    .bind(user_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    project.ok_or((
        StatusCode::NOT_FOUND,
        "Project not found or not writable".to_string(),
    ))
}

#[derive(Deserialize)]
pub struct DeviceLoginRequest {
    pub email: String,
    pub password: String,
    pub device_name: Option<String>,
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
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, is_admin FROM users WHERE email = ?",
    )
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((
        StatusCode::UNAUTHORIZED,
        "Invalid email or password".to_string(),
    ))?;

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
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
        "INSERT INTO device_tokens (id, user_id, name, token_hash, created_at) VALUES (?, ?, ?, ?, ?)",
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
        "SELECT id, username, email, password_hash, is_admin FROM users WHERE id = ?",
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

    sqlx::query("DELETE FROM device_tokens WHERE token_hash = ?")
        .bind(hash_token(&token))
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub entrypoint: String,
    pub role: String,
    pub updated_at: String,
}

pub async fn list_projects(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ProjectSummary>>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let owned = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, name, entrypoint, updated_at FROM projects WHERE owner_id = ? ORDER BY updated_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let shared = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT p.id, p.name, p.entrypoint, p.updated_at, c.role FROM projects p \
         INNER JOIN project_collaborators c ON c.project_id = p.id AND c.user_id = ? \
         ORDER BY p.updated_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut projects: Vec<ProjectSummary> = owned
        .into_iter()
        .map(|(id, name, entrypoint, updated_at)| ProjectSummary {
            id,
            name,
            entrypoint,
            role: "owner".to_string(),
            updated_at,
        })
        .collect();

    projects.extend(
        shared
            .into_iter()
            .map(|(id, name, entrypoint, updated_at, role)| ProjectSummary {
                id,
                name,
                entrypoint,
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

    let project_id = Uuid::new_v4().to_string();
    let entrypoint = payload
        .entrypoint
        .unwrap_or_else(|| "main.typ".to_string());

    let project = sqlx::query_as::<_, Project>(
        "INSERT INTO projects (id, owner_id, name, entrypoint) VALUES (?, ?, ?, ?) \
         RETURNING id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at",
    )
    .bind(&project_id)
    .bind(&user_id)
    .bind(payload.name.trim())
    .bind(&entrypoint)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ProjectSummary {
        id: project.id,
        name: project.name,
        entrypoint: project.entrypoint,
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

    let _ = sqlx::query("DELETE FROM project_files WHERE project_id = ?")
        .bind(&project_id)
        .execute(&state.db)
        .await;

    let result = sqlx::query("DELETE FROM projects WHERE id = ? AND owner_id = ?")
        .bind(&project_id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Project not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct ManifestEntry {
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
) -> Result<Vec<(String, String, Vec<u8>, String)>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, (String, String, Option<Vec<u8>>, Option<String>)>(
        "SELECT path, kind, content, updated_at FROM project_files WHERE project_id = ? ORDER BY path ASC",
    )
    .bind(project_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|(path, kind, content, updated_at)| {
            let raw = content.unwrap_or_default();
            let plain = if kind == "binary" {
                raw
            } else {
                decode_text_blob(&raw).into_bytes()
            };
            (path, kind, plain, updated_at.unwrap_or_default())
        })
        .collect())
}

pub async fn get_manifest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
) -> Result<Json<ProjectManifest>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    let project = owned_project(&state, &project_id, &user_id).await?;

    let files = plain_contents(&state, &project_id)
        .await?
        .into_iter()
        .map(|(path, kind, plain, updated_at)| ManifestEntry {
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
    owned_project(&state, &project_id, &user_id).await?;

    let row = sqlx::query_as::<_, (String, Option<Vec<u8>>)>(
        "SELECT kind, content FROM project_files WHERE project_id = ? AND path = ?",
    )
    .bind(&project_id)
    .bind(&query.path)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    let (kind, content) = row;
    let raw = content.unwrap_or_default();
    let plain = if kind == "binary" {
        raw
    } else {
        decode_text_blob(&raw).into_bytes()
    };

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
    owned_project(&state, &project_id, &user_id).await?;

    let incoming = match payload.encoding.as_deref() {
        Some("base64") => BASE64
            .decode(payload.content.as_bytes())
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid base64: {}", e)))?,
        _ => payload.content.clone().into_bytes(),
    };

    let existing = sqlx::query_as::<_, (String, Option<Vec<u8>>)>(
        "SELECT kind, content FROM project_files WHERE project_id = ? AND path = ?",
    )
    .bind(&project_id)
    .bind(&payload.path)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some((existing_kind, existing_content)) = &existing {
        let raw = existing_content.clone().unwrap_or_default();
        let plain = if existing_kind == "binary" {
            raw
        } else {
            decode_text_blob(&raw).into_bytes()
        };
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

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    sqlx::query(
        "INSERT INTO project_files (id, project_id, path, kind, content, mime_type, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?) \
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

    let _ = sqlx::query("UPDATE projects SET updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&project_id)
        .execute(&state.db)
        .await;

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
    owned_project(&state, &project_id, &user_id).await?;

    let result = sqlx::query("DELETE FROM project_files WHERE project_id = ? AND path = ?")
        .bind(&project_id)
        .bind(&query.path)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

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
    let project = owned_project(&state, &project_id, &user_id).await?;

    let files = plain_contents(&state, &project_id)
        .await?
        .into_iter()
        .map(|(path, kind, plain, _)| {
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
        "SELECT id, name, parent_id FROM folders WHERE owner_id = ? ORDER BY name ASC",
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
             WHERE owner_id = ? AND folder_id = ? ORDER BY updated_at DESC",
        )
        .bind(&user_id)
        .bind(folder_id)
        .fetch_all(&state.db)
        .await,
        None => sqlx::query_as::<_, (String, String, Option<String>, String)>(
            "SELECT id, title, folder_id, updated_at FROM documents \
             WHERE owner_id = ? AND folder_id IS NULL ORDER BY updated_at DESC",
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
         INNER JOIN collaborators c ON c.document_id = d.id AND c.user_id = ? \
         ORDER BY d.updated_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let projects = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT p.id, p.name, p.entrypoint, p.updated_at, c.role FROM projects p \
         INNER JOIN project_collaborators c ON c.project_id = p.id AND c.user_id = ? \
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
        "SELECT owner_id, title, content, public_role FROM documents WHERE id = ?",
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
            "SELECT role FROM collaborators WHERE document_id = ? AND user_id = ?",
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
    let (role, title, content) = document_role(&state, &id, &user_id).await?;

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
    let (role, _, current) = document_role(&state, &id, &user_id).await?;

    if role != "owner" && role != "editor" {
        return Err((StatusCode::FORBIDDEN, "Read-only access".to_string()));
    }

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

    sqlx::query("UPDATE documents SET content = ?, updated_at = ? WHERE id = ?")
        .bind(encode_text_blob(&payload.content))
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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
    let document_id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO documents (id, owner_id, folder_id, title, content) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&document_id)
    .bind(&user_id)
    .bind(&payload.folder_id)
    .bind(&payload.title)
    .bind(encode_text_blob(&payload.content))
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(DocumentContent {
        id: document_id,
        title: payload.title,
        role: "owner".to_string(),
        hash: content_hash(payload.content.as_bytes()),
        content: payload.content,
    }))
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
             WHERE owner_id = ? AND folder_id = ? ORDER BY name ASC",
        )
        .bind(&user_id)
        .bind(folder_id)
        .fetch_all(&state.db)
        .await,
        None => sqlx::query_as::<_, (String, String, String, Option<String>, String)>(
            "SELECT id, name, mime_type, folder_id, created_at FROM files \
             WHERE owner_id = ? AND folder_id IS NULL ORDER BY name ASC",
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
        "SELECT name, mime_type, data FROM files WHERE id = ? AND owner_id = ?",
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

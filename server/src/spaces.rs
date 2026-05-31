use axum::{
    extract::{Path, Query, State, Multipart},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use std::collections::HashMap;
use uuid::Uuid;
use yrs::{Doc, GetString, ReadTxn, StateVector, Text, Transact};
use yrs::updates::decoder::Decode;
use yrs::Update;

use crate::{
    compiler::ProjectInput,
    models::{
        CreateSpaceFileRequest, CreateSpaceRequest, Space, SpaceFile, UpdateSpaceFileRequest,
        UpdateSpaceRequest,
    },
    AppState,
};

const TEXT_NAME: &str = "typst";

pub fn encode_text_blob(text: &str) -> Vec<u8> {
    let doc = Doc::new();
    let handle = doc.get_or_insert_text(TEXT_NAME);
    handle.insert(&mut doc.transact_mut(), 0, text);
    let bytes = doc.transact().encode_state_as_update_v1(&StateVector::default());
    bytes
}

pub fn decode_text_blob(blob: &[u8]) -> String {
    let doc = Doc::new();
    if let Ok(update) = Update::decode_v1(blob) {
        doc.transact_mut().apply_update(update);
    }
    let handle = doc.get_or_insert_text(TEXT_NAME);
    let text = handle.get_string(&doc.transact());
    text
}

fn is_text_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    [".typ", ".toml", ".bib", ".csl", ".yml", ".yaml", ".json", ".md", ".txt", ".csv"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}

fn default_manifest(name: &str) -> String {
    format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nentrypoint = \"main.typ\"\nauthors = [\"Anonymous\"]\nlicense = \"MIT\"\ndescription = \"\"\n"
    )
}

fn slugify(name: &str) -> String {
    let slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let trimmed = slug.trim_matches('-').replace("--", "-");
    if trimmed.is_empty() {
        "my-space".to_string()
    } else {
        trimmed
    }
}

pub async fn space_role(
    state: &AppState,
    space_id: &str,
    user_id_opt: &Option<String>,
) -> Option<(Space, String)> {
    let space = sqlx::query_as::<_, Space>(
        "SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at FROM spaces WHERE id = ?"
    )
    .bind(space_id)
    .fetch_optional(&state.db)
    .await
    .ok()??;

    if let Some(uid) = user_id_opt {
        if &space.owner_id == uid {
            return Some((space, "owner".to_string()));
        }
        if let Ok(Some((role,))) = sqlx::query_as::<_, (String,)>(
            "SELECT role FROM space_collaborators WHERE space_id = ? AND user_id = ?",
        )
        .bind(space_id)
        .bind(uid)
        .fetch_optional(&state.db)
        .await
        {
            return Some((space, role));
        }
    }

    if let Some(pr) = space.public_role.clone() {
        if pr == "viewer" || pr == "editor" {
            return Some((space, pr));
        }
    }

    None
}

pub async fn load_local_packages(state: &AppState) -> HashMap<String, HashMap<String, Vec<u8>>> {
    let mut packages: HashMap<String, HashMap<String, Vec<u8>>> = HashMap::new();

    let rows = sqlx::query_as::<_, (String, String, String, Vec<u8>)>(
        "SELECT p.name, v.version, f.path, f.data \
         FROM package_files f \
         JOIN package_versions v ON v.id = f.version_id \
         JOIN packages p ON p.id = v.package_id",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for (name, version, path, data) in rows {
        let key = format!("{}:{}", name, version);
        packages.entry(key).or_default().insert(path, data);
    }

    packages
}

pub async fn assemble_project(
    state: &AppState,
    space: &Space,
    overrides: HashMap<String, String>,
) -> ProjectInput {
    let mut files: HashMap<String, Vec<u8>> = HashMap::new();

    // Account-level uploaded files (fonts, images) come first as a base layer so
    // they are available inside spaces; space files below override them by name.
    if let Ok(account_files) = sqlx::query_as::<_, (String, Vec<u8>)>(
        "SELECT name, data FROM files WHERE owner_id = ?",
    )
    .bind(&space.owner_id)
    .fetch_all(&state.db)
    .await
    {
        for (name, data) in account_files {
            files.insert(name, data);
        }
    }

    let rows = sqlx::query_as::<_, (String, String, Option<Vec<u8>>)>(
        "SELECT path, kind, content FROM space_files WHERE space_id = ?",
    )
    .bind(&space.id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for (path, kind, content) in rows {
        if let Some(live) = overrides.get(&path) {
            files.insert(path, live.clone().into_bytes());
        } else if kind == "binary" {
            files.insert(path, content.unwrap_or_default());
        } else {
            files.insert(path, decode_text_blob(&content.unwrap_or_default()).into_bytes());
        }
    }

    for (path, content) in overrides {
        files.entry(path).or_insert_with(|| content.into_bytes());
    }

    ProjectInput {
        entrypoint: space.entrypoint.clone(),
        files,
        packages: load_local_packages(state).await,
    }
}

#[derive(serde::Deserialize)]
pub struct ListSpacesQuery {
    pub folder_id: Option<String>,
}

pub async fn list_spaces(
    Query(query): Query<ListSpacesQuery>,
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<Space>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let spaces = if let Some(folder_id) = query.folder_id {
        sqlx::query_as::<_, Space>(
            "SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at FROM spaces WHERE owner_id = ? AND folder_id = ? ORDER BY updated_at DESC"
        )
        .bind(&user_id)
        .bind(&folder_id)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, Space>(
            "SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at FROM spaces WHERE owner_id = ? AND folder_id IS NULL ORDER BY updated_at DESC"
        )
        .bind(&user_id)
        .fetch_all(&state.db)
        .await
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(spaces))
}

pub async fn list_shared_spaces(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<Space>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let spaces = sqlx::query_as::<_, Space>(
        "SELECT s.id, s.owner_id, s.folder_id, s.name, s.entrypoint, s.thumbnail_svg, \
         s.public_role, s.created_at, s.updated_at, c.role as effective_role \
         FROM spaces s \
         INNER JOIN space_collaborators c ON c.space_id = s.id AND c.user_id = ? \
         ORDER BY s.updated_at DESC"
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(spaces))
}

pub async fn create_space(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<CreateSpaceRequest>,
) -> Result<Json<Space>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let space_id = Uuid::new_v4().to_string();

    let space = sqlx::query_as::<_, Space>(
        "INSERT INTO spaces (id, owner_id, folder_id, name, entrypoint) VALUES (?, ?, ?, ?, 'main.typ') RETURNING id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at"
    )
    .bind(&space_id)
    .bind(&user_id)
    .bind(&payload.folder_id)
    .bind(&payload.name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let seeds = [
        ("typst.toml", default_manifest(&slugify(&payload.name))),
        ("main.typ", "= New Space\n\nStart writing here.\n".to_string()),
    ];
    for (path, content) in seeds {
        let _ = sqlx::query(
            "INSERT INTO space_files (id, space_id, path, kind, content, mime_type) VALUES (?, ?, ?, 'text', ?, 'text/plain')"
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&space_id)
        .bind(path)
        .bind(encode_text_blob(&content))
        .execute(&state.db)
        .await;
    }

    Ok(Json(space))
}

pub async fn get_space(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<Space>, (StatusCode, String)> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());

    let (mut space, role) = space_role(&state, &id, &user_id_opt)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    space.effective_role = Some(role);
    Ok(Json(space))
}

pub async fn update_space(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    Json(payload): Json<UpdateSpaceRequest>,
) -> Result<Json<Space>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let mut space = sqlx::query_as::<_, Space>(
        "SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at FROM spaces WHERE id = ? AND owner_id = ?"
    )
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Space not found".to_string()))?;

    if let Some(name) = payload.name {
        space.name = name;
    }
    if let Some(entrypoint) = payload.entrypoint {
        space.entrypoint = entrypoint;
    }
    if let Some(folder_id) = payload.folder_id {
        space.folder_id = if folder_id.is_empty() { None } else { Some(folder_id) };
    }
    if let Some(public_role) = payload.public_role {
        space.public_role = if public_role == "none" || public_role.is_empty() {
            None
        } else {
            Some(public_role)
        };
    }

    let space = sqlx::query_as::<_, Space>(
        "UPDATE spaces SET name = ?, entrypoint = ?, folder_id = ?, public_role = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND owner_id = ? RETURNING id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at"
    )
    .bind(&space.name)
    .bind(&space.entrypoint)
    .bind(&space.folder_id)
    .bind(&space.public_role)
    .bind(&id)
    .bind(&user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(space))
}

pub async fn delete_space(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let _ = sqlx::query("DELETE FROM space_files WHERE space_id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    let result = sqlx::query("DELETE FROM spaces WHERE id = ? AND owner_id = ?")
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Space not found or unauthorized".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_space_files(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<SpaceFile>>, (StatusCode, String)> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());
    space_role(&state, &id, &user_id_opt)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let files = sqlx::query_as::<_, SpaceFile>(
        "SELECT id, space_id, path, kind, mime_type, created_at FROM space_files WHERE space_id = ? ORDER BY path ASC"
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(files))
}

pub async fn create_space_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    Json(payload): Json<CreateSpaceFileRequest>,
) -> Result<Json<SpaceFile>, (StatusCode, String)> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());
    let (_, role) = space_role(&state, &id, &user_id_opt)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if role == "viewer" {
        return Err((StatusCode::FORBIDDEN, "Read-only access".to_string()));
    }

    let kind = payload.kind.unwrap_or_else(|| "text".to_string());
    let content = payload.content.unwrap_or_default();
    let file_id = Uuid::new_v4().to_string();

    let file = sqlx::query_as::<_, SpaceFile>(
        "INSERT INTO space_files (id, space_id, path, kind, content, mime_type) VALUES (?, ?, ?, ?, ?, 'text/plain') RETURNING id, space_id, path, kind, mime_type, created_at"
    )
    .bind(&file_id)
    .bind(&id)
    .bind(&payload.path)
    .bind(&kind)
    .bind(encode_text_blob(&content))
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(file))
}

pub async fn upload_space_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());
    let (_, role) = space_role(&state, &id, &user_id_opt)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if role == "viewer" {
        return Err((StatusCode::FORBIDDEN, "Read-only access".to_string()));
    }

    let mut uploaded = vec![];

    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        let path = field.file_name().unwrap_or("unnamed").to_string();
        let mime_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data = field.bytes().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?.to_vec();

        let (kind, content) = if is_text_path(&path) {
            let text = String::from_utf8_lossy(&data).to_string();
            ("text", encode_text_blob(&text))
        } else {
            ("binary", data)
        };

        let _ = sqlx::query(
            "INSERT INTO space_files (id, space_id, path, kind, content, mime_type) VALUES (?, ?, ?, ?, ?, ?) \
             ON CONFLICT (space_id, path) DO UPDATE SET content = excluded.content, kind = excluded.kind, mime_type = excluded.mime_type"
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&id)
        .bind(&path)
        .bind(kind)
        .bind(content)
        .bind(&mime_type)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        uploaded.push(path);
    }

    Ok(Json(serde_json::json!({ "files": uploaded })))
}

pub async fn get_space_file(
    State(state): State<AppState>,
    Path((id, file_id)): Path<(String, String)>,
    jar: SignedCookieJar,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());
    space_role(&state, &id, &user_id_opt)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let file = sqlx::query_as::<_, (String, String, Option<Vec<u8>>)>(
        "SELECT kind, mime_type, content FROM space_files WHERE id = ? AND space_id = ?"
    )
    .bind(&file_id)
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    let (kind, mime_type, content) = file;
    let bytes = content.unwrap_or_default();

    if kind == "binary" {
        Ok(([(header::CONTENT_TYPE, mime_type)], bytes))
    } else {
        Ok(([(header::CONTENT_TYPE, "text/plain".to_string())], decode_text_blob(&bytes).into_bytes()))
    }
}

pub async fn update_space_file(
    State(state): State<AppState>,
    Path((id, file_id)): Path<(String, String)>,
    jar: SignedCookieJar,
    Json(payload): Json<UpdateSpaceFileRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());
    let (_, role) = space_role(&state, &id, &user_id_opt)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if role == "viewer" {
        return Err((StatusCode::FORBIDDEN, "Read-only access".to_string()));
    }

    let result = sqlx::query("UPDATE space_files SET path = ? WHERE id = ? AND space_id = ?")
        .bind(&payload.path)
        .bind(&file_id)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_space_file(
    State(state): State<AppState>,
    Path((id, file_id)): Path<(String, String)>,
    jar: SignedCookieJar,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());
    let (_, role) = space_role(&state, &id, &user_id_opt)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if role == "viewer" {
        return Err((StatusCode::FORBIDDEN, "Read-only access".to_string()));
    }

    let result = sqlx::query("DELETE FROM space_files WHERE id = ? AND space_id = ?")
        .bind(&file_id)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

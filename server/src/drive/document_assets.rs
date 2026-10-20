use std::collections::HashMap;

use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    sharing::access::{self, forbidden, AccessRole},
    models::{Document, File},
    drive::projects::decode_text_blob,
    AppState,
};
use crate::db::sql;

type ApiError = (StatusCode, String);

#[derive(Serialize)]
pub struct LinkedFile {
    #[serde(flatten)]
    file: File,
    shared_with_collaborators: bool,
}

struct StoredFile {
    file: File,
    data: Vec<u8>,
    linked_to_document: bool,
}

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn is_font_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".ttf") || lower.ends_with(".otf")
}

fn is_real_font(file: &StoredFile) -> bool {
    is_font_name(&file.file.name)
        && typst::text::Font::iter(typst::foundations::Bytes::new(file.data.clone())).next().is_some()
}

fn is_referenced(source: &str, file_name: &str) -> bool {
    source.contains(&format!("\"{file_name}\"")) || source.contains(&format!("\"/{file_name}\""))
}

fn session_user(jar: &SignedCookieJar) -> Option<String> {
    jar.get("session_user_id").map(|c| c.value().to_string())
}

async fn document_with_role(state: &AppState, document_id: &str, user_id: &Option<String>) -> Result<(Document, AccessRole), ApiError> {
    access::document_role(state, document_id, user_id).await.ok_or_else(forbidden)
}

async fn stored_files(state: &AppState, document: &Document) -> Result<Vec<StoredFile>, ApiError> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, String, String, String, Vec<u8>, i64, Option<String>, i64)>(
        sql("SELECT id, owner_id, document_id, folder_id, name, mime_type, created_at, data, size_bytes, storage_key, \
         CAST(CASE WHEN document_id = $1 OR id IN (SELECT file_id FROM document_shared_files WHERE document_id = $2) THEN 1 ELSE 0 END AS BIGINT) \
         FROM files WHERE document_id = $3 OR owner_id = $4 ORDER BY name ASC")
    )
    .bind(&document.id)
    .bind(&document.id)
    .bind(&document.id)
    .bind(&document.owner_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let loads = rows.into_iter().map(
        |(id, owner_id, document_id, folder_id, name, mime_type, created_at, data, size_bytes, storage_key, linked)| async move {
            StoredFile {
                file: File {
                    id,
                    owner_id,
                    document_id,
                    folder_id,
                    name,
                    mime_type,
                    created_at,
                    size_bytes: Some(size_bytes),
                    updated_at: None,
                },
                data: state.file_storage.load(data, storage_key).await.unwrap_or_default(),
                linked_to_document: linked != 0,
            }
        },
    );
    Ok(futures_util::future::join_all(loads).await)
}

async fn readable_by_collaborator(
    state: &AppState,
    user_id: &Option<String>,
    files: Vec<StoredFile>,
) -> Vec<StoredFile> {
    let mut readable_folders: HashMap<String, bool> = HashMap::new();
    let mut readable = Vec::new();
    for stored in files {
        let attached = stored.linked_to_document;
        let in_shared_folder = match &stored.file.folder_id {
            Some(folder_id) if user_id.is_some() => {
                if !readable_folders.contains_key(folder_id) {
                    let allowed = access::inherited_role(state, &Some(folder_id.clone()), &stored.file.owner_id, user_id).await.is_some();
                    readable_folders.insert(folder_id.clone(), allowed);
                }
                readable_folders[folder_id]
            }
            _ => false,
        };
        if attached || in_shared_folder || is_real_font(&stored) {
            readable.push(stored);
        }
    }
    readable
}

async fn files_for_role(
    state: &AppState,
    document: &Document,
    role: AccessRole,
    user_id: &Option<String>,
) -> Result<Vec<StoredFile>, ApiError> {
    let files = stored_files(state, document).await?;
    if role == AccessRole::Owner {
        return Ok(files);
    }
    Ok(readable_by_collaborator(state, user_id, files).await)
}

pub async fn compile_files(
    state: &AppState,
    document: &Document,
    role: AccessRole,
    user_id: &Option<String>,
) -> HashMap<String, Vec<u8>> {
    files_for_role(state, document, role, user_id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|stored| (stored.file.name, stored.data))
        .collect()
}

pub async fn list_assets(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<LinkedFile>>, ApiError> {
    let user_id = session_user(&jar);
    let (document, role) = document_with_role(&state, &document_id, &user_id).await?;
    let source = decode_text_blob(document.content.as_deref().unwrap_or_default());

    let linked = files_for_role(&state, &document, role, &user_id)
        .await?
        .into_iter()
        .filter(|stored| stored.linked_to_document || is_font_name(&stored.file.name) || is_referenced(&source, &stored.file.name))
        .map(|stored| LinkedFile {
            shared_with_collaborators: stored.linked_to_document || is_real_font(&stored),
            file: stored.file,
        })
        .collect();

    Ok(Json(linked))
}

pub async fn get_asset_data(
    State(state): State<AppState>,
    Path((document_id, file_id)): Path<(String, String)>,
    jar: SignedCookieJar,
) -> Result<impl IntoResponse, ApiError> {
    let user_id = session_user(&jar);
    let (document, role) = document_with_role(&state, &document_id, &user_id).await?;
    let stored = files_for_role(&state, &document, role, &user_id)
        .await?
        .into_iter()
        .find(|stored| stored.file.id == file_id)
        .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    Ok(([(header::CONTENT_TYPE, stored.file.mime_type)], stored.data))
}

async fn require_document_owner(state: &AppState, document_id: &str, jar: &SignedCookieJar) -> Result<Document, ApiError> {
    let user_id = session_user(jar);
    let (document, role) = document_with_role(state, document_id, &user_id).await?;
    if role != AccessRole::Owner {
        return Err((StatusCode::FORBIDDEN, "Only the document owner can share files with collaborators".to_string()));
    }
    Ok(document)
}

async fn link_file(state: &AppState, document_id: &str, file_id: &str) -> Result<(), ApiError> {
    sqlx::query(
        sql("INSERT INTO document_shared_files (document_id, file_id) VALUES ($1, $2) \
         ON CONFLICT (document_id, file_id) DO NOTHING")
    )
    .bind(document_id)
    .bind(file_id)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(())
}

pub async fn attach_asset(
    State(state): State<AppState>,
    Path((document_id, file_id)): Path<(String, String)>,
    jar: SignedCookieJar,
) -> Result<StatusCode, ApiError> {
    let document = require_document_owner(&state, &document_id, &jar).await?;

    let owned_file = sqlx::query_as::<_, (String,)>(sql("SELECT id FROM files WHERE id = $1 AND owner_id = $2"))
        .bind(&file_id)
        .bind(&document.owner_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;
    if owned_file.is_none() {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    link_file(&state, &document.id, &file_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn attach_unshared_assets(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<serde_json::Value>, ApiError> {
    let document = require_document_owner(&state, &document_id, &jar).await?;
    let source = decode_text_blob(document.content.as_deref().unwrap_or_default());

    let mut shared_count = 0;
    for stored in stored_files(&state, &document).await? {
        let needs_sharing = stored.file.owner_id == document.owner_id
            && !stored.linked_to_document
            && !is_real_font(&stored)
            && is_referenced(&source, &stored.file.name);
        if needs_sharing {
            link_file(&state, &document.id, &stored.file.id).await?;
            shared_count += 1;
        }
    }

    Ok(Json(serde_json::json!({ "shared": shared_count })))
}

pub async fn upload_asset(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    jar: SignedCookieJar,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = session_user(&jar);
    let (document, role) = document_with_role(&state, &document_id, &user_id).await?;
    if !role.can_edit() || user_id.is_none() {
        return Err(forbidden());
    }

    let mut uploaded_filename = String::new();
    let mut font_family = None;

    if let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        let file_name = field.file_name().unwrap_or("unnamed").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data = field.bytes().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?.to_vec();

        crate::accounts::storage::ensure_room(&state, &document.owner_id, data.len() as i64).await?;

        if is_font_name(&file_name) {
            if let Some(font) = typst::text::Font::iter(typst::foundations::Bytes::new(data.clone())).next() {
                font_family = Some(font.info().family.clone());
            }
        }

        let stored = state.file_storage.store(data).await?;
        sqlx::query(sql("INSERT INTO files (id, owner_id, document_id, folder_id, name, mime_type, data, size_bytes, storage_key) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"))
            .bind(Uuid::new_v4().to_string())
            .bind(&document.owner_id)
            .bind(&document.id)
            .bind(&document.folder_id)
            .bind(&file_name)
            .bind(&content_type)
            .bind(&stored.inline_data)
            .bind(stored.size_bytes)
            .bind(&stored.storage_key)
            .execute(&state.db)
            .await
            .map_err(internal_error)?;

        uploaded_filename = file_name;
    }

    Ok(Json(serde_json::json!({
        "filename": uploaded_filename,
        "font_family": font_family
    })))
}

pub async fn list_fonts(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<String>>, ApiError> {
    let user_id = session_user(&jar);
    let (document, role) = document_with_role(&state, &document_id, &user_id).await?;

    let mut families = std::collections::BTreeSet::new();
    for stored in files_for_role(&state, &document, role, &user_id).await? {
        if is_font_name(&stored.file.name) {
            for font in typst::text::Font::iter(typst::foundations::Bytes::new(stored.data)) {
                families.insert(font.info().family.clone());
            }
        }
    }

    Ok(Json(families.into_iter().collect()))
}

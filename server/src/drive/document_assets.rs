use std::collections::{BTreeSet, HashMap};

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::Response,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    sharing::access::{self, forbidden, AccessRole},
    models::{Document, File},
    drive::documents::saved_text,
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

struct FileEntry {
    file: File,
    linked_to_document: bool,
}

type ReadableFile = (FileEntry, Option<Vec<u8>>);

type FileEntryRow = (String, String, Option<String>, Option<String>, String, String, String, i64, i64);

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn not_found() -> ApiError {
    (StatusCode::NOT_FOUND, "File not found".to_string())
}

fn is_font_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".ttf") || lower.ends_with(".otf")
}

fn is_font_data(data: &[u8]) -> bool {
    typst::text::Font::iter(typst::foundations::Bytes::new(data.to_vec())).next().is_some()
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

async fn file_entries(state: &AppState, document: &Document) -> Result<Vec<FileEntry>, ApiError> {
    let rows = sqlx::query_as::<_, FileEntryRow>(
        sql("SELECT id, owner_id, document_id, folder_id, name, mime_type, created_at, size_bytes, \
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

    Ok(rows
        .into_iter()
        .map(|(id, owner_id, document_id, folder_id, name, mime_type, created_at, size_bytes, linked)| FileEntry {
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
            linked_to_document: linked != 0,
        })
        .collect())
}

async fn file_data(state: &AppState, file_id: &str) -> Result<Vec<u8>, ApiError> {
    let (data, storage_key) = sqlx::query_as::<_, (Vec<u8>, Option<String>)>(sql("SELECT data, storage_key FROM files WHERE id = $1"))
        .bind(file_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .ok_or_else(not_found)?;
    state.file_storage.load(data, storage_key).await
}

async fn is_real_font(state: &AppState, entry: &FileEntry, loaded_data: &Option<Vec<u8>>) -> bool {
    if !is_font_name(&entry.file.name) {
        return false;
    }
    match loaded_data {
        Some(data) => is_font_data(data),
        None => file_data(state, &entry.file.id).await.is_ok_and(|data| is_font_data(&data)),
    }
}

async fn readable_files(
    state: &AppState,
    document: &Document,
    role: AccessRole,
    user_id: &Option<String>,
) -> Result<Vec<ReadableFile>, ApiError> {
    let entries = file_entries(state, document).await?;
    if role == AccessRole::Owner {
        return Ok(entries.into_iter().map(|entry| (entry, None)).collect());
    }

    let mut readable_folders: HashMap<String, bool> = HashMap::new();
    let mut readable = Vec::new();
    for entry in entries {
        let in_shared_folder = match &entry.file.folder_id {
            Some(folder_id) if user_id.is_some() => {
                if !readable_folders.contains_key(folder_id) {
                    let allowed = access::inherited_role(state, &Some(folder_id.clone()), &entry.file.owner_id, user_id).await.is_some();
                    readable_folders.insert(folder_id.clone(), allowed);
                }
                readable_folders[folder_id]
            }
            _ => false,
        };
        if entry.linked_to_document || in_shared_folder {
            readable.push((entry, None));
        } else if is_font_name(&entry.file.name) {
            if let Ok(data) = file_data(state, &entry.file.id).await {
                if is_font_data(&data) {
                    readable.push((entry, Some(data)));
                }
            }
        }
    }
    Ok(readable)
}

async fn with_data(state: &AppState, files: Vec<ReadableFile>) -> Vec<(File, Vec<u8>)> {
    let missing_ids: Vec<String> = files
        .iter()
        .filter(|(_, data)| data.is_none())
        .map(|(entry, _)| entry.file.id.clone())
        .collect();
    let mut stored = crate::drive::file_data::load_many(state, &missing_ids).await;

    files
        .into_iter()
        .map(|(entry, loaded_data)| {
            let data = loaded_data.or_else(|| stored.remove(&entry.file.id)).unwrap_or_default();
            (entry.file, data)
        })
        .collect()
}

pub async fn compile_files(
    state: &AppState,
    document: &Document,
    role: AccessRole,
    user_id: &Option<String>,
) -> HashMap<String, Vec<u8>> {
    let Ok(files) = readable_files(state, document, role, user_id).await else {
        return HashMap::new();
    };
    let linked_ids: std::collections::HashSet<String> = files
        .iter()
        .filter(|(entry, _)| entry.linked_to_document)
        .map(|(entry, _)| entry.file.id.clone())
        .collect();
    let (linked, others): (Vec<_>, Vec<_>) = with_data(state, files).await.into_iter().partition(|(file, _)| linked_ids.contains(&file.id));
    others.into_iter().chain(linked).map(|(file, data)| (file.name, data)).collect()
}

pub async fn list_assets(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<LinkedFile>>, ApiError> {
    let user_id = session_user(&jar);
    let (document, role) = document_with_role(&state, &document_id, &user_id).await?;
    let source = saved_text(&state, &document.id).await;

    let mut linked = Vec::new();
    for (entry, loaded_data) in readable_files(&state, &document, role, &user_id).await? {
        let visible = entry.linked_to_document || is_font_name(&entry.file.name) || is_referenced(&source, &entry.file.name);
        if !visible {
            continue;
        }
        let shared_with_collaborators = entry.linked_to_document || is_real_font(&state, &entry, &loaded_data).await;
        linked.push(LinkedFile { file: entry.file, shared_with_collaborators });
    }

    Ok(Json(linked))
}

pub async fn get_asset_data(
    State(state): State<AppState>,
    Path((document_id, file_id)): Path<(String, String)>,
    jar: SignedCookieJar,
) -> Result<Response, ApiError> {
    let user_id = session_user(&jar);
    let (document, role) = document_with_role(&state, &document_id, &user_id).await?;
    let (entry, loaded_data) = readable_files(&state, &document, role, &user_id)
        .await?
        .into_iter()
        .find(|(entry, _)| entry.file.id == file_id)
        .ok_or_else(not_found)?;

    let data = match loaded_data {
        Some(data) => data,
        None => file_data(&state, &entry.file.id).await?,
    };
    Ok(crate::user_content::response(&entry.file.mime_type, data))
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
        return Err(not_found());
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
    let source = saved_text(&state, &document.id).await;

    let mut shared_count = 0;
    for entry in file_entries(&state, &document).await? {
        let needs_sharing = entry.file.owner_id == document.owner_id
            && !entry.linked_to_document
            && is_referenced(&source, &entry.file.name)
            && !is_real_font(&state, &entry, &None).await;
        if needs_sharing {
            link_file(&state, &document.id, &entry.file.id).await?;
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

    let font_files: Vec<ReadableFile> = readable_files(&state, &document, role, &user_id)
        .await?
        .into_iter()
        .filter(|(entry, _)| is_font_name(&entry.file.name))
        .collect();

    let mut families = BTreeSet::new();
    for (_, data) in with_data(&state, font_files).await {
        for font in typst::text::Font::iter(typst::foundations::Bytes::new(data)) {
            families.insert(font.info().family.clone());
        }
    }

    Ok(Json(families.into_iter().collect()))
}

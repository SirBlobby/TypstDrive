use axum::{
    extract::{Path, State, Query, Multipart},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    models::{File},
    AppState,
};
use crate::db::sql;

#[derive(Deserialize)]
pub struct ListFilesQuery {
    pub folder_id: Option<String>,
}

pub async fn list_files(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Query(query): Query<ListFilesQuery>,
) -> Result<Json<Vec<File>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let files = if let Some(folder_id) = query.folder_id {
        crate::sharing::access::require_folder_access(&state, &folder_id, &user_id).await?;
        sqlx::query_as::<_, File>(
            sql("SELECT id, owner_id, document_id, folder_id, name, mime_type, created_at, size_bytes, updated_at FROM files WHERE folder_id = $1 AND owner_id = (SELECT f.owner_id FROM folders f WHERE f.id = $2) ORDER BY name ASC")
        )
        .bind(&folder_id)
        .bind(&folder_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        sqlx::query_as::<_, File>(
            sql("SELECT id, owner_id, document_id, folder_id, name, mime_type, created_at, size_bytes, updated_at FROM files WHERE owner_id = $1 AND folder_id IS NULL ORDER BY name ASC")
        )
        .bind(&user_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    Ok(Json(files))
}

#[derive(Deserialize)]
pub struct UploadFileQuery {
    pub folder_id: Option<String>,
}

pub async fn upload_file_global(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Query(query): Query<UploadFileQuery>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    crate::sharing::access::require_owned_folder(&state, &query.folder_id, &user_id).await?;

    let mut uploaded_files = vec![];
    let mut font_families = vec![];

    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        let file_name = field.file_name().unwrap_or("unnamed").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data = field.bytes().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?.to_vec();

        crate::accounts::storage::ensure_room(&state, &user_id, data.len() as i64).await?;

        let file_id = Uuid::new_v4().to_string();

        let mut font_family = None;
        if file_name.to_lowercase().ends_with(".ttf") || file_name.to_lowercase().ends_with(".otf") {
            if let Some(font) = typst::text::Font::iter(typst::foundations::Bytes::new(data.clone())).next() {
                font_family = Some(font.info().family.clone());
            }
        }

        let stored = state.file_storage.store(data).await?;
        sqlx::query(sql("INSERT INTO files (id, owner_id, folder_id, name, mime_type, data, size_bytes, storage_key) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"))
            .bind(&file_id)
            .bind(&user_id)
            .bind(&query.folder_id)
            .bind(&file_name)
            .bind(&content_type)
            .bind(&stored.inline_data)
            .bind(stored.size_bytes)
            .bind(&stored.storage_key)
            .execute(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        uploaded_files.push(file_name);
        font_families.push(font_family);
    }

    Ok(Json(serde_json::json!({
        "files": uploaded_files,
        "font_families": font_families
    })))
}

pub async fn get_file_data(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let (owner_id, document_id, folder_id, mime_type, data, storage_key) = sqlx::query_as::<_, (String, Option<String>, Option<String>, String, Vec<u8>, Option<String>)>(
        sql("SELECT owner_id, document_id, folder_id, mime_type, data, storage_key FROM files WHERE id = $1")
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    if owner_id != user_id && !can_read_shared_file(&state, &id, &owner_id, &document_id, &folder_id, &user_id).await {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    let data = state.file_storage.load(data, storage_key).await?;
    Ok(crate::user_content::response(&mime_type, data))
}

pub async fn can_read_shared_file(
    state: &AppState,
    file_id: &str,
    file_owner_id: &str,
    document_id: &Option<String>,
    folder_id: &Option<String>,
    user_id: &str,
) -> bool {
    let user = Some(user_id.to_string());
    if crate::sharing::access::inherited_role(state, folder_id, file_owner_id, &user).await.is_some() {
        return true;
    }
    let mut linked_documents: Vec<String> = sqlx::query_as::<_, (String,)>(sql("SELECT document_id FROM document_shared_files WHERE file_id = $1"))
        .bind(file_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(id,)| id)
        .collect();
    linked_documents.extend(document_id.clone());
    for linked_document_id in linked_documents {
        if crate::sharing::access::document_role(state, &linked_document_id, &user).await.is_some() {
            return true;
        }
    }
    false
}

pub async fn delete_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let result = sqlx::query(sql("DELETE FROM files WHERE id = $1 AND owner_id = $2"))
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "File not found or unauthorized".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_fonts(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let files = sqlx::query_as::<_, (String, Vec<u8>, Option<String>)>(
        sql("SELECT name, data, storage_key FROM files WHERE owner_id = $1 AND (LOWER(name) LIKE '%.ttf' OR LOWER(name) LIKE '%.otf')")
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut families = std::collections::BTreeSet::new();
    for (_, data) in state.file_storage.load_named_skipping_failures(files).await {
        for font in typst::text::Font::iter(typst::foundations::Bytes::new(data)) {
            families.insert(font.info().family.clone());
        }
    }

    Ok(Json(families.into_iter().collect()))
}

#[derive(Deserialize)]
pub struct UpdateFileRequest {
    pub name: Option<String>,
    pub folder_id: Option<String>,
}

pub async fn update_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    Json(payload): Json<UpdateFileRequest>,
) -> Result<Json<File>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let mut file = sqlx::query_as::<_, File>(
        sql("SELECT id, owner_id, document_id, folder_id, name, mime_type, created_at FROM files WHERE id = $1 AND owner_id = $2")
    )
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    if let Some(new_name) = payload.name {
        let trimmed_name = new_name.trim();
        if trimmed_name.is_empty() || trimmed_name.contains('/') || trimmed_name.contains('\\') {
            return Err((StatusCode::BAD_REQUEST, "File name cannot be empty or contain slashes".to_string()));
        }
        file.name = trimmed_name.to_string();
    }
    if let Some(new_folder_id) = payload.folder_id {
        crate::sharing::access::require_owned_folder(&state, &Some(new_folder_id.clone()), &user_id).await?;
        if new_folder_id.is_empty() {
            file.folder_id = None;
        } else {
            file.folder_id = Some(new_folder_id);
        }
    }

    sqlx::query(sql("UPDATE files SET name = $1, folder_id = $2 WHERE id = $3 AND owner_id = $4"))
    .bind(&file.name)
    .bind(&file.folder_id)
    .bind(&id)
    .bind(&user_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let file = sqlx::query_as::<_, File>(sql("SELECT id, owner_id, document_id, folder_id, name, mime_type, created_at, size_bytes FROM files WHERE id = $1 AND owner_id = $2"))
    .bind(&id)
    .bind(&user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(file))
}

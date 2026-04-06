use axum::{
    extract::{Path, State, Query, Multipart},
    http::{StatusCode, header},
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
        sqlx::query_as::<_, File>(
            "SELECT id, owner_id, document_id, folder_id, name, mime_type, created_at FROM files WHERE owner_id = $1 AND folder_id = $2 ORDER BY name ASC"
        )
        .bind(&user_id)
        .bind(&folder_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        sqlx::query_as::<_, File>(
            "SELECT id, owner_id, document_id, folder_id, name, mime_type, created_at FROM files WHERE owner_id = $1 AND folder_id IS NULL ORDER BY name ASC"
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

    let mut uploaded_files = vec![];
    let mut font_families = vec![];

    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        let file_name = field.file_name().unwrap_or("unnamed").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data = field.bytes().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?.to_vec();

        let file_id = Uuid::new_v4().to_string();

        let mut font_family = None;
        if file_name.to_lowercase().ends_with(".ttf") || file_name.to_lowercase().ends_with(".otf") {
            if let Some(font) = typst::text::Font::iter(typst::foundations::Bytes::new(data.clone())).next() {
                font_family = Some(font.info().family.clone());
            }
        }

        sqlx::query("INSERT INTO files (id, owner_id, folder_id, name, mime_type, data) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(&file_id)
            .bind(&user_id)
            .bind(&query.folder_id)
            .bind(&file_name)
            .bind(&content_type)
            .bind(&data)
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

    let file = sqlx::query_as::<_, (String, Vec<u8>)>("SELECT mime_type, data FROM files WHERE id = $1 AND owner_id = $2")
        .bind(&id)
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some((mime_type, data)) = file {
        Ok((
            [(header::CONTENT_TYPE, mime_type)],
            data,
        ))
    } else {
        Err((StatusCode::NOT_FOUND, "File not found".to_string()))
    }
}

pub async fn delete_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let result = sqlx::query("DELETE FROM files WHERE id = $1 AND owner_id = $2")
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

    let files = sqlx::query_as::<_, (String,)>(
        "SELECT name FROM files WHERE owner_id = $1"
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut fonts = Vec::new();
    for (name,) in files {
        if name.to_lowercase().ends_with(".ttf") || name.to_lowercase().ends_with(".otf") {
            if let Some(stem) = std::path::Path::new(&name).file_stem() {
                if let Some(stem_str) = stem.to_str() {
                    fonts.push(stem_str.to_string());
                }
            }
        }
    }

    Ok(Json(fonts))
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
        "SELECT id, owner_id, document_id, folder_id, name, mime_type, created_at FROM files WHERE id = $1 AND owner_id = $2"
    )
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    if let Some(new_name) = payload.name {
        file.name = new_name;
    }
    if let Some(new_folder_id) = payload.folder_id {
        if new_folder_id.is_empty() {
            file.folder_id = None;
        } else {
            file.folder_id = Some(new_folder_id);
        }
    }

    let file = sqlx::query_as::<_, File>(
        "UPDATE files SET name = $1, folder_id = $2 WHERE id = $3 AND owner_id = $4 RETURNING id, owner_id, document_id, folder_id, name, mime_type, created_at"
    )
    .bind(&file.name)
    .bind(&file.folder_id)
    .bind(&id)
    .bind(&user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(file))
}

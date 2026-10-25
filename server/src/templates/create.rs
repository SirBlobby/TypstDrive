use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::files::{template_files, TemplateFile};
use super::index::{self, Template};
use super::{internal_error, ApiError};
use crate::compile::latex::TYPST_LANGUAGE;
use crate::db::sql;
use crate::drive::projects::{encode_text_blob, is_text_path};
use crate::sharing::access::require_owned_folder;
use crate::AppState;

const MAX_TITLE_LENGTH: usize = 200;

#[derive(Deserialize)]
pub struct CreateFromTemplateRequest {
    pub name: String,
    pub version: String,
    pub title: String,
    pub folder_id: Option<String>,
}

#[derive(Serialize)]
pub struct CreatedItem {
    pub kind: &'static str,
    pub id: String,
}

fn mime_type(path: &str) -> &'static str {
    let extension = path.rsplit('.').next().unwrap_or_default().to_ascii_lowercase();
    match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "pdf" => "application/pdf",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        _ if is_text_path(path) => "text/plain",
        _ => "application/octet-stream",
    }
}

fn clean_path(path: &str) -> &str {
    path.trim_start_matches("./")
}

async fn create_document(state: &AppState, user_id: &str, folder_id: &Option<String>, title: &str, file: &TemplateFile) -> Result<String, ApiError> {
    let document_id = Uuid::new_v4().to_string();
    let content = encode_text_blob(&String::from_utf8_lossy(&file.data));
    sqlx::query(sql("INSERT INTO documents (id, owner_id, folder_id, title, content, language) VALUES ($1, $2, $3, $4, $5, $6)"))
        .bind(&document_id)
        .bind(user_id)
        .bind(folder_id)
        .bind(title)
        .bind(&content)
        .bind(TYPST_LANGUAGE)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(document_id)
}

async fn create_project(
    state: &AppState,
    user_id: &str,
    folder_id: &Option<String>,
    title: &str,
    template: &Template,
    files: Vec<TemplateFile>,
) -> Result<String, ApiError> {
    let project_id = Uuid::new_v4().to_string();
    let mut transaction = state.db.begin().await.map_err(internal_error)?;

    sqlx::query(sql("INSERT INTO projects (id, owner_id, folder_id, name, entrypoint) VALUES ($1, $2, $3, $4, $5)"))
        .bind(&project_id)
        .bind(user_id)
        .bind(folder_id)
        .bind(title)
        .bind(clean_path(&template.entrypoint))
        .execute(&mut *transaction)
        .await
        .map_err(internal_error)?;

    for file in files {
        let (kind, content) = if is_text_path(&file.path) {
            ("text", encode_text_blob(&String::from_utf8_lossy(&file.data)))
        } else {
            ("binary", file.data)
        };
        sqlx::query(sql("INSERT INTO project_files (id, project_id, path, kind, content, mime_type) VALUES ($1, $2, $3, $4, $5, $6)"))
            .bind(Uuid::new_v4().to_string())
            .bind(&project_id)
            .bind(&file.path)
            .bind(kind)
            .bind(content)
            .bind(mime_type(&file.path))
            .execute(&mut *transaction)
            .await
            .map_err(internal_error)?;
    }

    transaction.commit().await.map_err(internal_error)?;
    Ok(project_id)
}

pub async fn create_from_template(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<CreateFromTemplateRequest>,
) -> Result<Json<CreatedItem>, ApiError> {
    let user_id = jar
        .get("session_user_id")
        .map(|cookie| cookie.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;
    require_owned_folder(&state, &payload.folder_id, &user_id).await?;

    let title: String = payload.title.trim().chars().take(MAX_TITLE_LENGTH).collect();
    if title.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Give the new document a name".to_string()));
    }

    let template = index::find(&payload.name, &payload.version).await?;
    let files = template_files(&template).await?;
    let total_bytes: i64 = files.iter().map(|file| file.data.len() as i64).sum();
    crate::accounts::storage::ensure_room(&state, &user_id, total_bytes).await?;

    let is_single_file = files.len() == 1 && files[0].path == clean_path(&template.entrypoint);
    let created = if is_single_file {
        let id = create_document(&state, &user_id, &payload.folder_id, &title, &files[0]).await?;
        CreatedItem { kind: "document", id }
    } else {
        let id = create_project(&state, &user_id, &payload.folder_id, &title, &template, files).await?;
        CreatedItem { kind: "project", id }
    };
    Ok(Json(created))
}

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use sqlx::FromRow;

use crate::api::authentication::{authenticate, ApiScope};
use crate::db::sql;
use crate::drive::projects::decode_text_blob;
use crate::AppState;

#[derive(Serialize, FromRow)]
pub struct DocumentSummary {
    pub id: String,
    pub title: String,
    pub folder_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct DocumentSource {
    #[serde(flatten)]
    pub summary: DocumentSummary,
    pub content: String,
}

#[derive(Serialize, FromRow)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub entrypoint: String,
    pub folder_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, FromRow)]
pub struct ProjectFileSummary {
    pub id: String,
    pub path: String,
    pub kind: String,
    pub mime_type: String,
}

#[derive(Serialize)]
pub struct ProjectDetail {
    #[serde(flatten)]
    pub summary: ProjectSummary,
    pub files: Vec<ProjectFileSummary>,
}

fn database_error(error: sqlx::Error) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
}

fn not_found(label: &str) -> Response {
    (StatusCode::NOT_FOUND, format!("{label} not found")).into_response()
}

pub async fn list_documents(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<DocumentSummary>>, Response> {
    let caller = authenticate(&state, &headers, ApiScope::Read).await?;
    sqlx::query_as::<_, DocumentSummary>(sql(
        "SELECT id, title, folder_id, created_at, updated_at FROM documents WHERE owner_id = $1 ORDER BY updated_at DESC",
    ))
    .bind(&caller.user_id)
    .fetch_all(&state.db)
    .await
    .map(Json)
    .map_err(database_error)
}

pub async fn get_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(document_id): Path<String>,
) -> Result<Json<DocumentSource>, Response> {
    let caller = authenticate(&state, &headers, ApiScope::Read).await?;
    let row = sqlx::query_as::<_, (String, String, Option<String>, String, String, Option<Vec<u8>>)>(sql(
        "SELECT id, title, folder_id, created_at, updated_at, content FROM documents WHERE id = $1 AND owner_id = $2",
    ))
    .bind(&document_id)
    .bind(&caller.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(database_error)?
    .ok_or_else(|| not_found("Document"))?;

    let (id, title, folder_id, created_at, updated_at, content) = row;
    Ok(Json(DocumentSource {
        summary: DocumentSummary { id, title, folder_id, created_at, updated_at },
        content: content.map(|blob| decode_text_blob(&blob)).unwrap_or_default(),
    }))
}

pub async fn list_projects(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<ProjectSummary>>, Response> {
    let caller = authenticate(&state, &headers, ApiScope::Read).await?;
    sqlx::query_as::<_, ProjectSummary>(sql(
        "SELECT id, name, entrypoint, folder_id, created_at, updated_at FROM projects WHERE owner_id = $1 ORDER BY updated_at DESC",
    ))
    .bind(&caller.user_id)
    .fetch_all(&state.db)
    .await
    .map(Json)
    .map_err(database_error)
}

pub async fn get_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
) -> Result<Json<ProjectDetail>, Response> {
    let caller = authenticate(&state, &headers, ApiScope::Read).await?;
    let summary = sqlx::query_as::<_, ProjectSummary>(sql(
        "SELECT id, name, entrypoint, folder_id, created_at, updated_at FROM projects WHERE id = $1 AND owner_id = $2",
    ))
    .bind(&project_id)
    .bind(&caller.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(database_error)?
    .ok_or_else(|| not_found("Project"))?;

    let files = sqlx::query_as::<_, ProjectFileSummary>(sql(
        "SELECT id, path, kind, mime_type FROM project_files WHERE project_id = $1 ORDER BY path ASC",
    ))
    .bind(&project_id)
    .fetch_all(&state.db)
    .await
    .map_err(database_error)?;

    Ok(Json(ProjectDetail { summary, files }))
}

pub async fn get_project_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, file_id)): Path<(String, String)>,
) -> Result<Response, Response> {
    let caller = authenticate(&state, &headers, ApiScope::Read).await?;
    let row = sqlx::query_as::<_, (String, String, Option<Vec<u8>>)>(sql(
        "SELECT pf.kind, pf.mime_type, pf.content FROM project_files pf INNER JOIN projects p ON p.id = pf.project_id \
         WHERE pf.id = $1 AND pf.project_id = $2 AND p.owner_id = $3",
    ))
    .bind(&file_id)
    .bind(&project_id)
    .bind(&caller.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(database_error)?
    .ok_or_else(|| not_found("File"))?;

    let (kind, mime_type, content) = row;
    let content = content.unwrap_or_default();
    if kind == "binary" {
        return Ok(([(axum::http::header::CONTENT_TYPE, mime_type)], content).into_response());
    }
    let text = decode_text_blob(&content);
    Ok(([(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8".to_string())], text).into_response())
}

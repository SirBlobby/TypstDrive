use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::{database_error, not_found};
use crate::api::authentication::{authenticate, ApiScope};
use crate::db::sql;
use crate::drive::comments::{document_comments, CommentView};
use crate::drive::people::PersonSummary;
use crate::drive::suggestions::{document_suggestions, SuggestionView};
use crate::drive::versions::{current_text, find_version, list_versions, VersionView};
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

#[derive(Serialize)]
pub struct VersionSummary {
    pub id: String,
    pub author: Option<PersonSummary>,
    pub editors: Vec<PersonSummary>,
    pub kind: String,
    pub label: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Deserialize)]
pub struct DocumentListQuery {
    pub folder_id: Option<String>,
}

impl From<VersionView> for VersionSummary {
    fn from(version: VersionView) -> Self {
        Self {
            id: version.id,
            author: version.author,
            editors: version.editors,
            kind: version.kind,
            label: version.label,
            created_at: version.created_at,
            updated_at: version.updated_at,
        }
    }
}

pub async fn owned_document(state: &AppState, account_id: &str, document_id: &str) -> Result<DocumentSummary, Response> {
    sqlx::query_as::<_, DocumentSummary>(sql(
        "SELECT id, title, folder_id, created_at, updated_at FROM documents WHERE id = $1 AND owner_id = $2",
    ))
    .bind(document_id)
    .bind(account_id)
    .fetch_optional(&state.db)
    .await
    .map_err(database_error)?
    .ok_or_else(|| not_found("Document"))
}

pub async fn list_documents(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DocumentListQuery>,
) -> Result<Json<Vec<DocumentSummary>>, Response> {
    let caller = authenticate(&state, &headers, ApiScope::Read).await?;
    let documents = match query.folder_id.filter(|id| !id.is_empty()) {
        Some(folder_id) => sqlx::query_as::<_, DocumentSummary>(sql(
            "SELECT id, title, folder_id, created_at, updated_at FROM documents \
             WHERE owner_id = $1 AND folder_id = $2 ORDER BY updated_at DESC",
        ))
        .bind(&caller.account_id)
        .bind(folder_id)
        .fetch_all(&state.db)
        .await,
        None => sqlx::query_as::<_, DocumentSummary>(sql(
            "SELECT id, title, folder_id, created_at, updated_at FROM documents WHERE owner_id = $1 ORDER BY updated_at DESC",
        ))
        .bind(&caller.account_id)
        .fetch_all(&state.db)
        .await,
    };
    documents.map(Json).map_err(database_error)
}

pub async fn get_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(document_id): Path<String>,
) -> Result<Json<DocumentSource>, Response> {
    let caller = authenticate(&state, &headers, ApiScope::Read).await?;
    let summary = owned_document(&state, &caller.account_id, &document_id).await?;
    let content = current_text(&state, &document_id).await.unwrap_or_default();
    Ok(Json(DocumentSource { summary, content }))
}

pub async fn get_document_versions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(document_id): Path<String>,
) -> Result<Json<Vec<VersionSummary>>, Response> {
    let caller = authenticate(&state, &headers, ApiScope::Read).await?;
    owned_document(&state, &caller.account_id, &document_id).await?;
    let versions = list_versions(&state, &document_id).await.map_err(database_error)?;
    Ok(Json(versions.into_iter().map(VersionSummary::from).collect()))
}

pub async fn get_document_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((document_id, version_id)): Path<(String, String)>,
) -> Result<Json<VersionView>, Response> {
    let caller = authenticate(&state, &headers, ApiScope::Read).await?;
    owned_document(&state, &caller.account_id, &document_id).await?;
    find_version(&state, &document_id, &version_id)
        .await
        .map_err(database_error)?
        .map(Json)
        .ok_or_else(|| not_found("Version"))
}

pub async fn get_document_comments(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(document_id): Path<String>,
) -> Result<Json<Vec<CommentView>>, Response> {
    let caller = authenticate(&state, &headers, ApiScope::Read).await?;
    owned_document(&state, &caller.account_id, &document_id).await?;
    document_comments(&state, &document_id).await.map(Json).map_err(database_error)
}

pub async fn get_document_suggestions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(document_id): Path<String>,
) -> Result<Json<Vec<SuggestionView>>, Response> {
    let caller = authenticate(&state, &headers, ApiScope::Read).await?;
    owned_document(&state, &caller.account_id, &document_id).await?;
    document_suggestions(&state, &document_id).await.map(Json).map_err(database_error)
}

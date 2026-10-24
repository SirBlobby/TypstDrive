pub mod admin;
pub mod cache;
pub mod manage;
pub mod policy;
pub mod serve;

use axum::http::StatusCode;
use serde::Serialize;

use crate::db::sql;
use crate::AppState;

pub type ApiError = (StatusCode, String);

const MIN_SLUG_LENGTH: usize = 3;
const MAX_SLUG_LENGTH: usize = 64;

macro_rules! link_query {
    ($tail:literal) => {
        concat!(
            "SELECT l.id, l.slug, l.document_id, l.project_id, COALESCE(d.title, p.name), l.view_count, l.last_viewed_at, l.created_at, u.username \
             FROM pdf_links l \
             LEFT JOIN documents d ON d.id = l.document_id \
             LEFT JOIN projects p ON p.id = l.project_id \
             LEFT JOIN users u ON u.id = COALESCE(d.owner_id, p.owner_id) ",
            $tail
        )
    };
}

type LinkRow = (String, String, Option<String>, Option<String>, Option<String>, i64, Option<String>, String, Option<String>);

pub fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PdfLinkKind {
    Document,
    Project,
}

impl PdfLinkKind {
    pub fn parse(value: &str) -> Result<Self, ApiError> {
        match value {
            "document" => Ok(Self::Document),
            "project" => Ok(Self::Project),
            _ => Err((StatusCode::BAD_REQUEST, "PDF links can only be made for documents and projects".to_string())),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Project => "project",
        }
    }
}

#[derive(Serialize)]
pub struct PdfLinkView {
    pub id: String,
    pub slug: String,
    pub kind: &'static str,
    pub item_id: String,
    pub item_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_username: Option<String>,
    pub view_count: i64,
    pub last_viewed_at: Option<String>,
    pub created_at: String,
}

fn view_from_row(row: LinkRow, include_owner: bool) -> PdfLinkView {
    let (id, slug, document_id, project_id, item_name, view_count, last_viewed_at, created_at, owner_username) = row;
    let (kind, item_id) = match (document_id, project_id) {
        (Some(document_id), _) => (PdfLinkKind::Document, document_id),
        (None, project_id) => (PdfLinkKind::Project, project_id.unwrap_or_default()),
    };
    PdfLinkView {
        id,
        slug,
        kind: kind.as_str(),
        item_id,
        item_name: item_name.unwrap_or_default(),
        owner_username: if include_owner { owner_username } else { None },
        view_count,
        last_viewed_at,
        created_at,
    }
}

pub fn clean_slug(value: &str) -> Result<String, ApiError> {
    let slug = value.trim().to_lowercase();
    let invalid = |message: &str| -> Result<String, ApiError> { Err((StatusCode::BAD_REQUEST, message.to_string())) };
    if slug.len() < MIN_SLUG_LENGTH || slug.len() > MAX_SLUG_LENGTH {
        return invalid(&format!("Short links must be {MIN_SLUG_LENGTH} to {MAX_SLUG_LENGTH} characters long"));
    }
    if !slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return invalid("Short links can only use letters, numbers and hyphens");
    }
    if slug.starts_with('-') || slug.ends_with('-') {
        return invalid("Short links cannot start or end with a hyphen");
    }
    Ok(slug)
}

pub fn suggested_slug(name: &str) -> String {
    let mut slug = String::new();
    for character in name.to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
        } else if !slug.ends_with('-') && !slug.is_empty() {
            slug.push('-');
        }
    }
    let slug: String = slug.trim_matches('-').chars().take(MAX_SLUG_LENGTH).collect();
    let slug = slug.trim_end_matches('-').to_string();
    if slug.len() < MIN_SLUG_LENGTH { "my-document".to_string() } else { slug }
}

pub async fn link_for_item(state: &AppState, kind: PdfLinkKind, item_id: &str) -> Result<Option<PdfLinkView>, ApiError> {
    let query = match kind {
        PdfLinkKind::Document => sql(link_query!("WHERE l.document_id = $1")),
        PdfLinkKind::Project => sql(link_query!("WHERE l.project_id = $1")),
    };
    let row = sqlx::query_as::<_, LinkRow>(query)
        .bind(item_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(row.map(|row| view_from_row(row, false)))
}

pub async fn links_for_owner(state: &AppState, owner_id: &str) -> Result<Vec<PdfLinkView>, ApiError> {
    let rows = sqlx::query_as::<_, LinkRow>(sql(link_query!("WHERE COALESCE(d.owner_id, p.owner_id) = $1 ORDER BY l.created_at DESC")))
        .bind(owner_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(rows.into_iter().map(|row| view_from_row(row, false)).collect())
}

pub async fn all_links(state: &AppState) -> Result<Vec<PdfLinkView>, ApiError> {
    let rows = sqlx::query_as::<_, LinkRow>(sql(link_query!("ORDER BY l.created_at DESC")))
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(rows.into_iter().map(|row| view_from_row(row, true)).collect())
}

pub mod admin;
pub mod cache;
pub mod manage;
pub mod policy;
pub mod presentation;
pub mod serve;
pub mod target;

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
            "SELECT l.id AS id, l.slug AS slug, l.document_id AS document_id, l.project_id AS project_id, \
             COALESCE(d.title, p.name) AS item_name, l.created_at AS created_at, u.username AS owner_username, \
             l.pdf_enabled AS pdf_enabled, l.view_count AS pdf_views, l.last_viewed_at AS pdf_last_viewed_at, \
             l.slides_enabled AS slides_enabled, l.slides_view_count AS slides_views, l.slides_last_viewed_at AS slides_last_viewed_at, \
             l.speaker_enabled AS speaker_enabled, l.speaker_view_count AS speaker_views, l.speaker_last_viewed_at AS speaker_last_viewed_at, \
             CAST(CASE WHEN l.speaker_password_hash IS NULL THEN 0 ELSE 1 END AS BIGINT) AS has_speaker_password \
             FROM public_links l \
             LEFT JOIN documents d ON d.id = l.document_id \
             LEFT JOIN projects p ON p.id = l.project_id \
             LEFT JOIN users u ON u.id = COALESCE(d.owner_id, p.owner_id) ",
            $tail
        )
    };
}

#[derive(sqlx::FromRow)]
struct LinkRow {
    id: String,
    slug: String,
    document_id: Option<String>,
    project_id: Option<String>,
    item_name: Option<String>,
    created_at: String,
    owner_username: Option<String>,
    pdf_enabled: i64,
    pdf_views: i64,
    pdf_last_viewed_at: Option<String>,
    slides_enabled: i64,
    slides_views: i64,
    slides_last_viewed_at: Option<String>,
    speaker_enabled: i64,
    speaker_views: i64,
    speaker_last_viewed_at: Option<String>,
    has_speaker_password: i64,
}

pub fn internal_error(error: sqlx::Error) -> ApiError {
    crate::server_errors::hidden(error)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LinkKind {
    Document,
    Project,
}

impl LinkKind {
    pub fn parse(value: &str) -> Result<Self, ApiError> {
        match value {
            "document" => Ok(Self::Document),
            "project" => Ok(Self::Project),
            _ => Err((StatusCode::BAD_REQUEST, "Public links can only be made for documents and projects".to_string())),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Project => "project",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Format {
    Pdf,
    Slides,
    Speaker,
}

impl Format {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pdf => "PDF",
            Self::Slides => "slides",
            Self::Speaker => "speaker view",
        }
    }
}

#[derive(Serialize)]
pub struct FormatView {
    pub enabled: bool,
    pub views: i64,
    pub last_viewed_at: Option<String>,
}

#[derive(Serialize)]
pub struct PublicLinkView {
    pub id: String,
    pub slug: String,
    pub kind: &'static str,
    pub item_id: String,
    pub item_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_username: Option<String>,
    pub created_at: String,
    pub pdf: FormatView,
    pub slides: FormatView,
    pub speaker: FormatView,
    pub has_speaker_password: bool,
}

fn format_view(enabled: i64, views: i64, last_viewed_at: Option<String>) -> FormatView {
    FormatView { enabled: enabled != 0, views, last_viewed_at }
}

fn view_from_row(row: LinkRow, include_owner: bool) -> PublicLinkView {
    let (kind, item_id) = match (row.document_id, row.project_id) {
        (Some(document_id), _) => (LinkKind::Document, document_id),
        (None, project_id) => (LinkKind::Project, project_id.unwrap_or_default()),
    };
    PublicLinkView {
        id: row.id,
        slug: row.slug,
        kind: kind.as_str(),
        item_id,
        item_name: row.item_name.unwrap_or_default(),
        owner_username: if include_owner { row.owner_username } else { None },
        created_at: row.created_at,
        pdf: format_view(row.pdf_enabled, row.pdf_views, row.pdf_last_viewed_at),
        slides: format_view(row.slides_enabled, row.slides_views, row.slides_last_viewed_at),
        speaker: format_view(row.speaker_enabled, row.speaker_views, row.speaker_last_viewed_at),
        has_speaker_password: row.has_speaker_password != 0,
    }
}

pub fn clean_slug(value: &str) -> Result<String, ApiError> {
    let slug = value.trim().to_lowercase();
    let invalid = |message: &str| -> Result<String, ApiError> { Err((StatusCode::BAD_REQUEST, message.to_string())) };
    if slug.len() < MIN_SLUG_LENGTH || slug.len() > MAX_SLUG_LENGTH {
        return invalid(&format!("Link names must be {MIN_SLUG_LENGTH} to {MAX_SLUG_LENGTH} characters long"));
    }
    if !slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return invalid("Link names can only use letters, numbers and hyphens");
    }
    if slug.starts_with('-') || slug.ends_with('-') {
        return invalid("Link names cannot start or end with a hyphen");
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

pub async fn link_for_item(state: &AppState, kind: LinkKind, item_id: &str) -> Result<Option<PublicLinkView>, ApiError> {
    let query = match kind {
        LinkKind::Document => sql(link_query!("WHERE l.document_id = $1")),
        LinkKind::Project => sql(link_query!("WHERE l.project_id = $1")),
    };
    let row = sqlx::query_as::<_, LinkRow>(query)
        .bind(item_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(row.map(|row| view_from_row(row, false)))
}

pub async fn links_for_owner(state: &AppState, owner_id: &str) -> Result<Vec<PublicLinkView>, ApiError> {
    let rows = sqlx::query_as::<_, LinkRow>(sql(link_query!("WHERE COALESCE(d.owner_id, p.owner_id) = $1 ORDER BY l.created_at DESC")))
        .bind(owner_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(rows.into_iter().map(|row| view_from_row(row, false)).collect())
}

pub async fn all_links(state: &AppState) -> Result<Vec<PublicLinkView>, ApiError> {
    let rows = sqlx::query_as::<_, LinkRow>(sql(link_query!("ORDER BY l.created_at DESC")))
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(rows.into_iter().map(|row| view_from_row(row, true)).collect())
}

use std::collections::HashMap;

use super::policy::load_policy;
use super::Format;
use crate::compile::compiler::ProjectInput;
use crate::compile::latex;
use crate::db::{sql, timestamp_now};
use crate::drive::versions::current_text;
use crate::models::Project;
use crate::sharing::access::{self, AccessRole};
use crate::AppState;

#[derive(sqlx::FromRow)]
pub struct LinkTarget {
    pub id: String,
    pub slug: String,
    pub document_id: Option<String>,
    pub project_id: Option<String>,
    pub title: Option<String>,
    pub pdf_enabled: i64,
    pub slides_enabled: i64,
    pub speaker_enabled: i64,
    pub speaker_password_hash: Option<String>,
}

impl LinkTarget {
    pub fn offers(&self, format: Format) -> bool {
        match format {
            Format::Pdf => self.pdf_enabled != 0,
            Format::Slides => self.slides_enabled != 0,
            Format::Speaker => self.speaker_enabled != 0 && self.speaker_password_hash.is_some(),
        }
    }
}

pub fn clean_requested_slug(slug: &str) -> String {
    slug.strip_suffix(".pdf").unwrap_or(slug).to_lowercase()
}

pub async fn find(state: &AppState, slug: &str, format: Format) -> Option<LinkTarget> {
    if !load_policy(state).await.allows(format) {
        return None;
    }
    let target = sqlx::query_as::<_, LinkTarget>(sql(
        "SELECT l.id AS id, l.slug AS slug, l.document_id AS document_id, l.project_id AS project_id, \
         COALESCE(d.title, p.name) AS title, l.pdf_enabled AS pdf_enabled, l.slides_enabled AS slides_enabled, \
         l.speaker_enabled AS speaker_enabled, l.speaker_password_hash AS speaker_password_hash \
         FROM public_links l \
         LEFT JOIN documents d ON d.id = l.document_id \
         LEFT JOIN projects p ON p.id = l.project_id \
         WHERE l.slug = $1",
    ))
    .bind(clean_requested_slug(slug))
    .fetch_optional(&state.db)
    .await
    .ok()??;
    target.offers(format).then_some(target)
}

async fn document_input(state: &AppState, document_id: &str) -> Option<ProjectInput> {
    let (owner_id,) = sqlx::query_as::<_, (String,)>(sql("SELECT owner_id FROM documents WHERE id = $1 AND deleted_at IS NULL"))
        .bind(document_id)
        .fetch_optional(&state.db)
        .await
        .ok()??;
    let (document, _) = access::document_role(state, document_id, &Some(owner_id)).await?;
    let text = current_text(state, &document.id).await.unwrap_or_default();
    let files = crate::drive::document_assets::compile_files(state, &document, AccessRole::Viewer, &None).await;
    let mut input = latex::document_input(&document, text, files);
    crate::packages::imports::attach_imported(state, &mut input, "").await;
    Some(input)
}

async fn project_input(state: &AppState, project_id: &str) -> Option<ProjectInput> {
    let project = sqlx::query_as::<_, Project>(
        sql("SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at FROM projects WHERE id = $1 AND deleted_at IS NULL"),
    )
    .bind(project_id)
    .fetch_optional(&state.db)
    .await
    .ok()??;
    let reader = crate::drive::projects::ProjectReader::Collaborator(None);
    Some(crate::drive::projects::assemble_project(state, &project, HashMap::new(), reader).await)
}

pub async fn compile_input(state: &AppState, target: &LinkTarget) -> Option<ProjectInput> {
    match (&target.document_id, &target.project_id) {
        (Some(document_id), _) => document_input(state, document_id).await,
        (None, Some(project_id)) => project_input(state, project_id).await,
        (None, None) => None,
    }
}

pub async fn record_view(state: &AppState, link_id: &str, format: Format) {
    let statement = match format {
        Format::Pdf => "UPDATE public_links SET view_count = view_count + 1, last_viewed_at = $1 WHERE id = $2",
        Format::Slides => "UPDATE public_links SET slides_view_count = slides_view_count + 1, slides_last_viewed_at = $1 WHERE id = $2",
        Format::Speaker => "UPDATE public_links SET speaker_view_count = speaker_view_count + 1, speaker_last_viewed_at = $1 WHERE id = $2",
    };
    let _ = sqlx::query(sql(statement)).bind(timestamp_now()).bind(link_id).execute(&state.db).await;
}

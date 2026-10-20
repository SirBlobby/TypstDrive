use serde::Serialize;
use uuid::Uuid;

use crate::db::{sql, sql_owned};
use crate::drive::people::{load_people, PersonSummary};
use super::merge::{can_merge, latest_version, merge_into};
use crate::realtime::live_documents;
use crate::AppState;

pub const KIND_MANUAL: &str = "manual";
pub const KIND_AUTO: &str = "auto";
pub const KIND_RESTORE: &str = "restore";

#[derive(Serialize)]
pub struct VersionView {
    pub id: String,
    pub document_id: String,
    pub author: Option<PersonSummary>,
    pub editors: Vec<PersonSummary>,
    pub content: String,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub kind: String,
    pub label: Option<String>,
}

type VersionRow = (String, String, String, String, String, Option<String>, String, Option<String>, Option<String>);

const VERSION_COLUMNS: &str = "SELECT id, document_id, user_id, content, created_at, updated_at, kind, label, editor_ids FROM document_versions";

fn split_ids(editor_ids: &Option<String>) -> Vec<String> {
    editor_ids
        .as_deref()
        .unwrap_or_default()
        .split(',')
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect()
}

async fn to_views(state: &AppState, rows: Vec<VersionRow>) -> Vec<VersionView> {
    let mut all_ids: Vec<String> = Vec::new();
    for (_, _, user_id, _, _, _, _, _, editor_ids) in &rows {
        all_ids.push(user_id.clone());
        all_ids.extend(split_ids(editor_ids));
    }
    let directory = load_people(state, &all_ids).await;

    rows.into_iter()
        .map(|(id, document_id, user_id, content, created_at, updated_at, kind, label, editor_ids)| {
            let mut editor_list = split_ids(&editor_ids);
            if editor_list.is_empty() {
                editor_list.push(user_id.clone());
            }
            VersionView {
                id,
                document_id,
                author: directory.get(&user_id).cloned(),
                editors: editor_list.iter().filter_map(|editor_id| directory.get(editor_id).cloned()).collect(),
                content,
                created_at,
                updated_at,
                kind,
                label,
            }
        })
        .collect()
}

pub async fn list_versions(state: &AppState, document_id: &str) -> Result<Vec<VersionView>, sqlx::Error> {
    let rows = sqlx::query_as::<_, VersionRow>(sql_owned(format!(
        "{VERSION_COLUMNS} WHERE document_id = $1 ORDER BY created_at DESC, id DESC"
    )))
    .bind(document_id)
    .fetch_all(&state.db)
    .await?;
    Ok(to_views(state, rows).await)
}

pub async fn find_version(state: &AppState, document_id: &str, version_id: &str) -> Result<Option<VersionView>, sqlx::Error> {
    let row = sqlx::query_as::<_, VersionRow>(sql_owned(format!("{VERSION_COLUMNS} WHERE id = $1 AND document_id = $2")))
        .bind(version_id)
        .bind(document_id)
        .fetch_optional(&state.db)
        .await?;
    Ok(match row {
        Some(row) => to_views(state, vec![row]).await.pop(),
        None => None,
    })
}

pub async fn current_text(state: &AppState, document_id: &str) -> Option<String> {
    let room_id = live_documents::document_room(document_id);
    if let Some(text) = live_documents::live_text(state, &room_id).await {
        return Some(text);
    }
    let (content,) = sqlx::query_as::<_, (Option<Vec<u8>>,)>(sql("SELECT content FROM documents WHERE id = $1"))
        .bind(document_id)
        .fetch_optional(&state.db)
        .await
        .ok()??;
    Some(crate::drive::projects::decode_text_blob(content.as_deref().unwrap_or_default()))
}

pub struct NewVersion<'a> {
    pub document_id: &'a str,
    pub author_id: &'a str,
    pub content: &'a str,
    pub kind: &'a str,
    pub label: Option<&'a str>,
}

pub async fn save_version(state: &AppState, version: NewVersion<'_>) -> Result<Option<String>, sqlx::Error> {
    let latest = latest_version(state, version.document_id).await;
    if latest.as_ref().is_some_and(|latest| latest.content == version.content) {
        return Ok(None);
    }

    let room_id = live_documents::document_room(version.document_id);
    let mut editors = crate::drive::activity::take_editors_since_version(&room_id);
    editors.insert(version.author_id.to_string());

    if let Some(latest) = latest.filter(|latest| can_merge(latest, &version)) {
        merge_into(state, &latest, &version, editors).await?;
        return Ok(Some(latest.id));
    }
    let editor_ids = editors.into_iter().collect::<Vec<_>>().join(",");

    let version_id = Uuid::new_v4().to_string();
    sqlx::query(sql(
        "INSERT INTO document_versions (id, document_id, user_id, content, created_at, kind, label, editor_ids) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    ))
    .bind(&version_id)
    .bind(version.document_id)
    .bind(version.author_id)
    .bind(version.content)
    .bind(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string())
    .bind(version.kind)
    .bind(version.label)
    .bind(&editor_ids)
    .execute(&state.db)
    .await?;
    super::retention::enforce_limit(state, version.document_id).await?;
    Ok(Some(version_id))
}

pub async fn save_when_leaving(state: &AppState, document_id: &str, user_id: &str) {
    let Some(content) = current_text(state, document_id).await else {
        return;
    };
    let version = NewVersion { document_id, author_id: user_id, content: &content, kind: KIND_AUTO, label: None };
    if let Err(error) = save_version(state, version).await {
        tracing::warn!("could not save a version of {document_id}: {error}");
    }
}

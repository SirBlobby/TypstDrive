use std::collections::BTreeSet;

use chrono::{Duration, NaiveDateTime, Utc};

use crate::db::sql;
use crate::AppState;

use super::history::{NewVersion, KIND_AUTO};

const MERGE_WINDOW_MINUTES: i64 = 15;

pub struct LatestVersion {
    pub id: String,
    pub content: String,
    kind: String,
    label: Option<String>,
    created_at: String,
    editor_ids: Option<String>,
}

pub async fn latest_version(state: &AppState, document_id: &str) -> Option<LatestVersion> {
    let (id, content, kind, label, created_at, editor_ids) =
        sqlx::query_as::<_, (String, String, String, Option<String>, String, Option<String>)>(sql(
            "SELECT id, content, kind, label, created_at, editor_ids FROM document_versions WHERE document_id = $1 ORDER BY created_at DESC, id DESC LIMIT 1",
        ))
        .bind(document_id)
        .fetch_optional(&state.db)
        .await
        .ok()??;
    Some(LatestVersion { id, content, kind, label, created_at, editor_ids })
}

fn parse_timestamp(value: &str) -> Option<NaiveDateTime> {
    ["%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%d %H:%M:%S"]
        .iter()
        .find_map(|format| NaiveDateTime::parse_from_str(value, format).ok())
}

fn is_plain_auto_save(kind: &str, label: Option<&str>) -> bool {
    kind == KIND_AUTO && label.is_none()
}

pub fn can_merge(latest: &LatestVersion, version: &NewVersion<'_>) -> bool {
    if !is_plain_auto_save(version.kind, version.label) || !is_plain_auto_save(&latest.kind, latest.label.as_deref()) {
        return false;
    }
    let merge_window = Duration::minutes(MERGE_WINDOW_MINUTES);
    parse_timestamp(&latest.created_at).is_some_and(|started_at| Utc::now().naive_utc() - started_at < merge_window)
}

pub async fn merge_into(state: &AppState, latest: &LatestVersion, version: &NewVersion<'_>, new_editors: BTreeSet<String>) -> Result<(), sqlx::Error> {
    let mut editors: BTreeSet<String> = latest
        .editor_ids
        .as_deref()
        .unwrap_or_default()
        .split(',')
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect();
    editors.extend(new_editors);
    let editor_ids = editors.into_iter().collect::<Vec<_>>().join(",");

    sqlx::query(sql("UPDATE document_versions SET content = $1, user_id = $2, editor_ids = $3, updated_at = $4 WHERE id = $5"))
        .bind(version.content)
        .bind(version.author_id)
        .bind(&editor_ids)
        .bind(Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string())
        .bind(&latest.id)
        .execute(&state.db)
        .await?;
    Ok(())
}

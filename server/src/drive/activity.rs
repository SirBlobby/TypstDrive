use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use crate::db::{sql, timestamp_now};
use crate::AppState;

static LIVE_EDITORS: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn note_live_edit(room_id: &str, user_id: &str) {
    let mut editors = LIVE_EDITORS.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    editors.insert(room_id.to_string(), user_id.to_string());
}

pub fn live_editor(room_id: &str) -> Option<String> {
    let editors = LIVE_EDITORS.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    editors.get(room_id).cloned()
}

pub async fn mark_document_edited(state: &AppState, document_id: &str, editor_id: Option<&str>) {
    let _ = sqlx::query(sql(
        "UPDATE documents SET last_edited_by = COALESCE($1, last_edited_by), updated_at = $2 WHERE id = $3",
    ))
    .bind(editor_id)
    .bind(timestamp_now())
    .bind(document_id)
    .execute(&state.db)
    .await;
}

pub async fn mark_project_edited(state: &AppState, project_id: &str, editor_id: Option<&str>) {
    let _ = sqlx::query(sql(
        "UPDATE projects SET last_edited_by = COALESCE($1, last_edited_by), updated_at = $2 WHERE id = $3",
    ))
    .bind(editor_id)
    .bind(timestamp_now())
    .bind(project_id)
    .execute(&state.db)
    .await;
}

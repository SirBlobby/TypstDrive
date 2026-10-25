use std::collections::{HashMap, HashSet};

use futures_util::future::join_all;

use crate::db::{sql, sql_owned};
use crate::AppState;

pub async fn load_many(state: &AppState, file_ids: &[String]) -> HashMap<String, Vec<u8>> {
    let mut unique_ids = file_ids.to_vec();
    unique_ids.sort();
    unique_ids.dedup();
    if unique_ids.is_empty() {
        return HashMap::new();
    }

    let placeholders = (1..=unique_ids.len()).map(|index| format!("${index}")).collect::<Vec<_>>().join(", ");
    let statement = format!("SELECT id, data, storage_key FROM files WHERE id IN ({placeholders})");
    let mut query = sqlx::query_as::<_, (String, Vec<u8>, Option<String>)>(sql_owned(statement));
    for id in &unique_ids {
        query = query.bind(id);
    }
    let rows = query.fetch_all(&state.db).await.unwrap_or_default();

    let loads = rows.into_iter().map(|(id, inline_data, storage_key)| async move {
        state.file_storage.load(inline_data, storage_key).await.ok().map(|data| (id, data))
    });
    join_all(loads).await.into_iter().flatten().collect()
}

pub fn is_font_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".ttf") || lower.ends_with(".otf")
}

pub async fn account_files(state: &AppState, owner_id: &str, sources: &[&str], skipped_names: &HashSet<String>) -> HashMap<String, Vec<u8>> {
    let entries = sqlx::query_as::<_, (String, String)>(sql("SELECT id, name FROM files WHERE owner_id = $1 AND deleted_at IS NULL"))
        .bind(owner_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let needed: Vec<(String, String)> = entries
        .into_iter()
        .filter(|(_, name)| !skipped_names.contains(name))
        .filter(|(_, name)| is_font_name(name) || sources.iter().any(|source| source.contains(name.as_str())))
        .collect();
    let ids: Vec<String> = needed.iter().map(|(id, _)| id.clone()).collect();
    let mut loaded = load_many(state, &ids).await;

    needed
        .into_iter()
        .filter_map(|(id, name)| loaded.remove(&id).map(|data| (name, data)))
        .collect()
}

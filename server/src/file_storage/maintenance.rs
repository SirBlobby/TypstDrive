use std::collections::HashSet;
use std::time::Duration;

use futures_util::StreamExt;
use object_store::path::Path as ObjectPath;

use super::FileStorage;
use crate::db::sql;
use crate::AppState;

const SWEEP_INTERVAL: Duration = Duration::from_secs(60 * 60);
const ORPHAN_GRACE_PERIOD_MINUTES: i64 = 60;
const MOVE_BATCH_SIZE: usize = 20;

pub fn spawn(state: AppState) {
    if !state.file_storage.uses_object_storage() {
        return;
    }
    tokio::spawn(async move {
        loop {
            move_database_files(&state).await;
            remove_orphaned_objects(&state).await;
            tokio::time::sleep(SWEEP_INTERVAL).await;
        }
    });
}

async fn move_database_files(state: &AppState) {
    let mut moved = 0;
    loop {
        let batch = sqlx::query_as::<_, (String, Vec<u8>)>(sql(
            "SELECT id, data FROM files WHERE storage_key IS NULL AND size_bytes > 0 LIMIT $1",
        ))
        .bind(MOVE_BATCH_SIZE as i64)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        if batch.is_empty() {
            break;
        }
        let batch_length = batch.len();

        for (file_id, data) in batch {
            let stored = match state.file_storage.store(data).await {
                Ok(stored) => stored,
                Err(_) => return,
            };
            let updated = sqlx::query(sql("UPDATE files SET data = $1, storage_key = $2 WHERE id = $3 AND storage_key IS NULL"))
                .bind(&stored.inline_data)
                .bind(&stored.storage_key)
                .bind(&file_id)
                .execute(&state.db)
                .await;
            if updated.is_err() {
                return;
            }
            moved += 1;
        }
        if batch_length < MOVE_BATCH_SIZE {
            break;
        }
    }
    if moved > 0 {
        tracing::info!("Moved {moved} uploaded files from the database to object storage");
    }
}

async fn referenced_keys(state: &AppState) -> Option<HashSet<String>> {
    sqlx::query_as::<_, (String,)>(sql("SELECT DISTINCT storage_key FROM files WHERE storage_key IS NOT NULL"))
        .fetch_all(&state.db)
        .await
        .ok()
        .map(|rows| rows.into_iter().map(|(key,)| key).collect())
}

async fn remove_orphaned_objects(state: &AppState) {
    let storage: &FileStorage = &state.file_storage;
    let Some(remote) = &storage.remote else {
        return;
    };
    let Some(in_use) = referenced_keys(state).await else {
        return;
    };

    let cutoff = chrono::Utc::now() - chrono::Duration::minutes(ORPHAN_GRACE_PERIOD_MINUTES);
    let prefix = ObjectPath::from(FileStorage::files_prefix(remote).as_str());
    let mut orphans = Vec::new();
    let mut listing = remote.store.list(Some(&prefix));
    while let Some(entry) = listing.next().await {
        let Ok(object) = entry else {
            return;
        };
        if object.last_modified < cutoff && !in_use.contains(&object.location.to_string()) {
            orphans.push(object.location);
        }
    }
    drop(listing);

    for location in &orphans {
        if let Err(error) = remote.store.delete(location).await {
            tracing::warn!("Could not delete unused object {location}: {error}");
        }
    }
    if !orphans.is_empty() {
        tracing::info!("Removed {} unused objects from object storage", orphans.len());
    }
}

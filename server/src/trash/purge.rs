use std::time::Duration;

use super::{internal_error, policy, ApiError, TrashKind};
use crate::db::sql;
use crate::AppState;

const PURGE_INTERVAL: Duration = Duration::from_secs(60 * 60);

pub async fn hard_delete(state: &AppState, kind: TrashKind, item_id: &str) -> Result<(), ApiError> {
    let query = match kind {
        TrashKind::Document => sql("DELETE FROM documents WHERE id = $1"),
        TrashKind::Project => sql("DELETE FROM projects WHERE id = $1"),
        TrashKind::File => sql("DELETE FROM files WHERE id = $1"),
        TrashKind::Folder => sql("DELETE FROM folders WHERE id = $1"),
    };
    sqlx::query(query).bind(item_id).execute(&state.db).await.map_err(internal_error)?;
    Ok(())
}

pub async fn purge_entry(state: &AppState, trash_id: &str, kind: TrashKind, item_id: &str) -> Result<(), ApiError> {
    hard_delete(state, kind, item_id).await?;
    sqlx::query(sql("DELETE FROM trash_items WHERE id = $1"))
        .bind(trash_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(())
}

pub async fn purge_entries(state: &AppState, entries: Vec<(String, String, String)>) -> usize {
    let mut purged = 0;
    for (trash_id, kind, item_id) in entries {
        let Some(kind) = TrashKind::parse(&kind) else {
            continue;
        };
        match purge_entry(state, &trash_id, kind, &item_id).await {
            Ok(()) => purged += 1,
            Err((_, message)) => tracing::warn!("Could not empty trash item {trash_id}: {message}"),
        }
    }
    remove_orphans(state).await;
    purged
}

pub async fn remove_orphans(state: &AppState) {
    let queries = [
        sql("DELETE FROM trash_items WHERE kind = 'document' AND item_id NOT IN (SELECT id FROM documents)"),
        sql("DELETE FROM trash_items WHERE kind = 'project' AND item_id NOT IN (SELECT id FROM projects)"),
        sql("DELETE FROM trash_items WHERE kind = 'file' AND item_id NOT IN (SELECT id FROM files)"),
        sql("DELETE FROM trash_items WHERE kind = 'folder' AND item_id NOT IN (SELECT id FROM folders)"),
    ];
    for query in queries {
        if let Err(error) = sqlx::query(query).execute(&state.db).await {
            tracing::warn!("Could not tidy the trash: {error}");
        }
    }
}

async fn purge_expired(state: &AppState) {
    let retention_days = policy::retention_days(state).await;
    let cutoff = (chrono::Utc::now() - chrono::Duration::days(retention_days)).format("%Y-%m-%d %H:%M:%S").to_string();
    let expired = sqlx::query_as::<_, (String, String, String)>(sql("SELECT id, kind, item_id FROM trash_items WHERE deleted_at <= $1"))
        .bind(&cutoff)
        .fetch_all(&state.db)
        .await;
    match expired {
        Ok(entries) if !entries.is_empty() => {
            let purged = purge_entries(state, entries).await;
            tracing::info!("Emptied {purged} expired trash items");
        }
        Ok(_) => {}
        Err(error) => tracing::warn!("Could not check the trash for expired items: {error}"),
    }
}

pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        loop {
            purge_expired(&state).await;
            tokio::time::sleep(PURGE_INTERVAL).await;
        }
    });
}

use std::collections::HashMap;

use super::{internal_error, ApiError};
use crate::db::sql;
use crate::AppState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotEntry {
    pub blob_sha: String,
    pub content_hash: String,
}

pub type Snapshot = HashMap<String, SnapshotEntry>;

pub struct SnapshotUpdate {
    pub saved: Vec<(String, SnapshotEntry)>,
    pub removed: Vec<String>,
}

pub async fn load(state: &AppState, project_id: &str) -> Result<Snapshot, ApiError> {
    let rows = sqlx::query_as::<_, (String, String, String)>(sql("SELECT path, blob_sha, content_hash FROM git_link_files WHERE project_id = $1"))
        .bind(project_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(rows
        .into_iter()
        .map(|(path, blob_sha, content_hash)| (path, SnapshotEntry { blob_sha, content_hash }))
        .collect())
}

pub async fn apply(state: &AppState, project_id: &str, update: &SnapshotUpdate, base_commit: &str, synced_at: &str) -> Result<(), ApiError> {
    let mut transaction = state.db.begin().await.map_err(internal_error)?;
    let paths = update.removed.iter().chain(update.saved.iter().map(|(path, _)| path));
    for path in paths {
        sqlx::query(sql("DELETE FROM git_link_files WHERE project_id = $1 AND path = $2"))
            .bind(project_id)
            .bind(path)
            .execute(&mut *transaction)
            .await
            .map_err(internal_error)?;
    }
    for (path, entry) in &update.saved {
        sqlx::query(sql("INSERT INTO git_link_files (project_id, path, blob_sha, content_hash) VALUES ($1, $2, $3, $4)"))
            .bind(project_id)
            .bind(path)
            .bind(&entry.blob_sha)
            .bind(&entry.content_hash)
            .execute(&mut *transaction)
            .await
            .map_err(internal_error)?;
    }
    sqlx::query(sql("UPDATE git_links SET base_commit = $1, last_synced_at = $2 WHERE project_id = $3"))
        .bind(base_commit)
        .bind(synced_at)
        .bind(project_id)
        .execute(&mut *transaction)
        .await
        .map_err(internal_error)?;
    transaction.commit().await.map_err(internal_error)
}

pub async fn clear(state: &AppState, project_id: &str) -> Result<(), ApiError> {
    sqlx::query(sql("DELETE FROM git_link_files WHERE project_id = $1"))
        .bind(project_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(())
}

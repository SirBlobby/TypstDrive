use axum::http::StatusCode;
use uuid::Uuid;

use super::{internal_error, policy, purge, ApiError, TrashKind};
use crate::db::{sql, timestamp_now};
use crate::sharing::access::{revalidate_resource, Resource};
use crate::AppState;

struct LiveItem {
    name: String,
    folder_id: Option<String>,
}

async fn live_item(state: &AppState, kind: TrashKind, item_id: &str, owner_id: &str) -> Result<Option<LiveItem>, ApiError> {
    let query = match kind {
        TrashKind::Document => sql("SELECT title, folder_id FROM documents WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL"),
        TrashKind::Project => sql("SELECT name, folder_id FROM projects WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL"),
        TrashKind::File => sql("SELECT name, folder_id FROM files WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL"),
        TrashKind::Folder => sql("SELECT name, parent_id FROM folders WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL"),
    };
    let row = sqlx::query_as::<_, (String, Option<String>)>(query)
        .bind(item_id)
        .bind(owner_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(row.map(|(name, folder_id)| LiveItem { name, folder_id }))
}

async fn live_folder_tree(state: &AppState, folder_id: &str) -> Result<Vec<String>, ApiError> {
    let rows = sqlx::query_as::<_, (String,)>(sql(
        "WITH RECURSIVE tree(id) AS ( \
             SELECT id FROM folders WHERE id = $1 AND deleted_at IS NULL \
             UNION ALL SELECT f.id FROM folders f INNER JOIN tree ON f.parent_id = tree.id WHERE f.deleted_at IS NULL \
         ) SELECT id FROM tree",
    ))
    .bind(folder_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

async fn affected_resources(state: &AppState, kind: TrashKind, item_id: &str, owner_id: &str) -> Result<Vec<Resource>, ApiError> {
    Ok(match kind {
        TrashKind::Document => vec![Resource::Document(item_id.to_string())],
        TrashKind::Project => vec![Resource::Project(item_id.to_string())],
        TrashKind::File => Vec::new(),
        TrashKind::Folder => {
            let documents = crate::drive::folders::items_in_folder_tree(state, "documents", item_id, owner_id).await?;
            let projects = crate::drive::folders::items_in_folder_tree(state, "projects", item_id, owner_id).await?;
            documents.into_iter().map(Resource::Document).chain(projects.into_iter().map(Resource::Project)).collect()
        }
    })
}

async fn mark_rows(state: &AppState, kind: TrashKind, item_id: &str, trash_id: &str, deleted_at: &str) -> Result<(), ApiError> {
    let folder_ids = match kind {
        TrashKind::Folder => live_folder_tree(state, item_id).await?,
        _ => Vec::new(),
    };
    let mut transaction = state.db.begin().await.map_err(internal_error)?;

    let single_row = match kind {
        TrashKind::Document => Some(sql("UPDATE documents SET deleted_at = $1, trash_id = $2 WHERE id = $3 AND deleted_at IS NULL")),
        TrashKind::Project => Some(sql("UPDATE projects SET deleted_at = $1, trash_id = $2 WHERE id = $3 AND deleted_at IS NULL")),
        TrashKind::File => Some(sql("UPDATE files SET deleted_at = $1, trash_id = $2 WHERE id = $3 AND deleted_at IS NULL")),
        TrashKind::Folder => None,
    };
    if let Some(query) = single_row {
        sqlx::query(query).bind(deleted_at).bind(trash_id).bind(item_id).execute(&mut *transaction).await.map_err(internal_error)?;
    }

    let folder_queries = [
        sql("UPDATE folders SET deleted_at = $1, trash_id = $2 WHERE id = $3 AND deleted_at IS NULL"),
        sql("UPDATE documents SET deleted_at = $1, trash_id = $2 WHERE folder_id = $3 AND deleted_at IS NULL"),
        sql("UPDATE projects SET deleted_at = $1, trash_id = $2 WHERE folder_id = $3 AND deleted_at IS NULL"),
        sql("UPDATE files SET deleted_at = $1, trash_id = $2 WHERE folder_id = $3 AND deleted_at IS NULL"),
    ];
    for folder_id in &folder_ids {
        for query in folder_queries {
            sqlx::query(query).bind(deleted_at).bind(trash_id).bind(folder_id).execute(&mut *transaction).await.map_err(internal_error)?;
        }
    }

    transaction.commit().await.map_err(internal_error)
}

pub async fn move_to_trash(state: &AppState, kind: TrashKind, item_id: &str, owner_id: &str, deleted_by: &str) -> Result<(), ApiError> {
    let item = live_item(state, kind, item_id, owner_id)
        .await?
        .ok_or((StatusCode::NOT_FOUND, "Item not found or unauthorized".to_string()))?;
    let resources = affected_resources(state, kind, item_id, owner_id).await?;

    if policy::retention_days(state).await == 0 {
        purge::hard_delete(state, kind, item_id).await?;
    } else {
        let trash_id = Uuid::new_v4().to_string();
        let deleted_at = timestamp_now();
        mark_rows(state, kind, item_id, &trash_id, &deleted_at).await?;
        sqlx::query(sql(
            "INSERT INTO trash_items (id, owner_id, kind, item_id, name, original_folder_id, deleted_by, deleted_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        ))
        .bind(&trash_id)
        .bind(owner_id)
        .bind(kind.as_str())
        .bind(item_id)
        .bind(&item.name)
        .bind(&item.folder_id)
        .bind(deleted_by)
        .bind(&deleted_at)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    }

    for resource in resources {
        revalidate_resource(state, resource).await;
    }
    Ok(())
}

async fn move_to_root_if_parent_gone(state: &AppState, kind: TrashKind, item_id: &str) -> Result<(), ApiError> {
    let parent_query = match kind {
        TrashKind::Document => sql("SELECT folder_id FROM documents WHERE id = $1"),
        TrashKind::Project => sql("SELECT folder_id FROM projects WHERE id = $1"),
        TrashKind::File => sql("SELECT folder_id FROM files WHERE id = $1"),
        TrashKind::Folder => sql("SELECT parent_id FROM folders WHERE id = $1"),
    };
    let parent = sqlx::query_as::<_, (Option<String>,)>(parent_query)
        .bind(item_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .and_then(|(parent,)| parent);
    let Some(parent) = parent else {
        return Ok(());
    };

    let parent_alive = sqlx::query_as::<_, (String,)>(sql("SELECT id FROM folders WHERE id = $1 AND deleted_at IS NULL"))
        .bind(&parent)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .is_some();
    if parent_alive {
        return Ok(());
    }

    let detach_query = match kind {
        TrashKind::Document => sql("UPDATE documents SET folder_id = NULL WHERE id = $1"),
        TrashKind::Project => sql("UPDATE projects SET folder_id = NULL WHERE id = $1"),
        TrashKind::File => sql("UPDATE files SET folder_id = NULL WHERE id = $1"),
        TrashKind::Folder => sql("UPDATE folders SET parent_id = NULL WHERE id = $1"),
    };
    sqlx::query(detach_query).bind(item_id).execute(&state.db).await.map_err(internal_error)?;
    Ok(())
}

pub async fn restore_rows(state: &AppState, trash_id: &str, kind: TrashKind, item_id: &str) -> Result<(), ApiError> {
    let mut transaction = state.db.begin().await.map_err(internal_error)?;
    let restore_queries = [
        sql("UPDATE folders SET deleted_at = NULL, trash_id = NULL WHERE trash_id = $1"),
        sql("UPDATE documents SET deleted_at = NULL, trash_id = NULL WHERE trash_id = $1"),
        sql("UPDATE projects SET deleted_at = NULL, trash_id = NULL WHERE trash_id = $1"),
        sql("UPDATE files SET deleted_at = NULL, trash_id = NULL WHERE trash_id = $1"),
    ];
    for query in restore_queries {
        sqlx::query(query).bind(trash_id).execute(&mut *transaction).await.map_err(internal_error)?;
    }
    sqlx::query(sql("DELETE FROM trash_items WHERE id = $1"))
        .bind(trash_id)
        .execute(&mut *transaction)
        .await
        .map_err(internal_error)?;
    transaction.commit().await.map_err(internal_error)?;
    move_to_root_if_parent_gone(state, kind, item_id).await
}

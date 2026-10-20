use axum::http::StatusCode;
use sqlx::{Any, Transaction};
use uuid::Uuid;

use crate::{sharing::members::ShareKind, AppState};
use crate::db::{sql, sql_owned};

type ApiError = (StatusCode, String);

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

async fn single_size(state: &AppState, statement: &'static str, binds: &[&str]) -> Result<i64, ApiError> {
    let mut query = sqlx::query_as::<_, (i64,)>(sql(statement));
    for value in binds {
        query = query.bind(value.to_string());
    }
    let (size,) = query.fetch_one(&state.db).await.map_err(internal_error)?;
    Ok(size)
}

pub async fn folder_tree(state: &AppState, folder_id: &str, owner_id: &str) -> Result<Vec<String>, ApiError> {
    let rows = sqlx::query_as::<_, (String,)>(
        sql("WITH RECURSIVE tree(id, owner_id) AS ( \
             SELECT id, owner_id FROM folders WHERE id = $1 AND owner_id = $2 \
             UNION ALL SELECT f.id, f.owner_id FROM folders f INNER JOIN tree ON f.parent_id = tree.id AND f.owner_id = tree.owner_id \
         ) SELECT id FROM tree")
    )
    .bind(folder_id)
    .bind(owner_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

pub async fn item_size(state: &AppState, kind: ShareKind, item_id: &str, owner_id: &str) -> Result<i64, ApiError> {
    match kind {
        ShareKind::Document => {
            let document = single_size(
                state,
                "SELECT CAST(COALESCE(SUM(LENGTH(content)), 0) AS BIGINT) FROM documents WHERE id = $1 AND owner_id = $2",
                &[item_id, owner_id],
            )
            .await?;
            let files = single_size(
                state,
                "SELECT CAST(COALESCE(SUM(size_bytes), 0) AS BIGINT) FROM files WHERE owner_id = $1 AND \
                 (document_id = $2 OR id IN (SELECT file_id FROM document_shared_files WHERE document_id = $3))",
                &[owner_id, item_id, item_id],
            )
            .await?;
            Ok(document + files)
        }
        ShareKind::Project => {
            single_size(
                state,
                "SELECT CAST(COALESCE(SUM(LENGTH(f.content)), 0) AS BIGINT) FROM project_files f \
                 INNER JOIN projects p ON p.id = f.project_id WHERE p.id = $1 AND p.owner_id = $2",
                &[item_id, owner_id],
            )
            .await
        }
        ShareKind::Folder => {
            let mut total = 0;
            for folder_id in folder_tree(state, item_id, owner_id).await? {
                total += single_size(
                    state,
                    "SELECT CAST(COALESCE(SUM(LENGTH(content)), 0) AS BIGINT) FROM documents WHERE folder_id = $1 AND owner_id = $2",
                    &[&folder_id, owner_id],
                )
                .await?;
                total += single_size(
                    state,
                    "SELECT CAST(COALESCE(SUM(size_bytes), 0) AS BIGINT) FROM files WHERE folder_id = $1 AND owner_id = $2",
                    &[&folder_id, owner_id],
                )
                .await?;
                total += single_size(
                    state,
                    "SELECT CAST(COALESCE(SUM(LENGTH(f.content)), 0) AS BIGINT) FROM project_files f \
                     INNER JOIN projects p ON p.id = f.project_id WHERE p.folder_id = $1 AND p.owner_id = $2",
                    &[&folder_id, owner_id],
                )
                .await?;
            }
            Ok(total)
        }
    }
}

async fn run(transaction: &mut Transaction<'static, Any>, statement: &'static str, binds: &[&str]) -> Result<(), ApiError> {
    let mut query = sqlx::query(sql(statement));
    for value in binds {
        query = query.bind(value.to_string());
    }
    query.execute(&mut **transaction).await.map_err(internal_error)?;
    Ok(())
}

async fn swap_membership(
    transaction: &mut Transaction<'static, Any>,
    kind: ShareKind,
    item_id: &str,
    previous_owner_id: &str,
    new_owner_id: &str,
) -> Result<(), ApiError> {
    let remove_new_owner = format!(
        "DELETE FROM {} WHERE {} = $1 AND user_id = $2",
        kind.collaborator_table(),
        kind.item_column()
    );
    sqlx::query(sql_owned(remove_new_owner))
        .bind(item_id)
        .bind(new_owner_id)
        .execute(&mut **transaction)
        .await
        .map_err(internal_error)?;

    let keep_previous_owner = format!(
        "INSERT INTO {table} (id, {column}, user_id, role) VALUES ($1, $2, $3, 'editor') \
         ON CONFLICT ({column}, user_id) DO UPDATE SET role = 'editor'",
        table = kind.collaborator_table(),
        column = kind.item_column()
    );
    sqlx::query(sql_owned(keep_previous_owner))
        .bind(Uuid::new_v4().to_string())
        .bind(item_id)
        .bind(previous_owner_id)
        .execute(&mut **transaction)
        .await
        .map_err(internal_error)?;
    Ok(())
}

async fn copy_file(
    transaction: &mut Transaction<'static, Any>,
    file_id: &str,
    new_owner_id: &str,
    document_id: &str,
) -> Result<(), ApiError> {
    let copy_id = Uuid::new_v4().to_string();
    run(
        transaction,
        "INSERT INTO files (id, owner_id, document_id, folder_id, name, mime_type, data, size_bytes, storage_key) \
         SELECT $1, $2, $3, NULL, name, mime_type, data, size_bytes, storage_key FROM files WHERE id = $4",
        &[copy_id.as_str(), new_owner_id, document_id, file_id],
    )
    .await
}

async fn file_ids(transaction: &mut Transaction<'static, Any>, statement: &'static str, binds: &[&str]) -> Result<Vec<String>, ApiError> {
    let mut query = sqlx::query_as::<_, (String,)>(sql(statement));
    for value in binds {
        query = query.bind(value.to_string());
    }
    let rows = query.fetch_all(&mut **transaction).await.map_err(internal_error)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

async fn move_document(
    transaction: &mut Transaction<'static, Any>,
    document_id: &str,
    previous_owner_id: &str,
    new_owner_id: &str,
) -> Result<(), ApiError> {
    let attached_and_used_elsewhere = file_ids(
        transaction,
        "SELECT DISTINCT f.id FROM files f INNER JOIN document_shared_files l ON l.file_id = f.id \
         WHERE f.document_id = $1 AND f.owner_id = $2 AND l.document_id != $3",
        &[document_id, previous_owner_id, document_id],
    )
    .await?;
    let linked_only = file_ids(
        transaction,
        "SELECT f.id FROM files f INNER JOIN document_shared_files l ON l.file_id = f.id \
         WHERE l.document_id = $1 AND f.owner_id = $2 AND (f.document_id IS NULL OR f.document_id != $3)",
        &[document_id, previous_owner_id, document_id],
    )
    .await?;

    run(transaction, "UPDATE documents SET owner_id = $1, folder_id = NULL WHERE id = $2", &[new_owner_id, document_id]).await?;

    for file_id in &attached_and_used_elsewhere {
        copy_file(transaction, file_id, new_owner_id, document_id).await?;
        run(transaction, "UPDATE files SET document_id = NULL WHERE id = $1", &[file_id.as_str()]).await?;
    }
    run(
        transaction,
        "UPDATE files SET owner_id = $1, folder_id = NULL WHERE document_id = $2 AND owner_id = $3",
        &[new_owner_id, document_id, previous_owner_id],
    )
    .await?;
    for file_id in &linked_only {
        copy_file(transaction, file_id, new_owner_id, document_id).await?;
    }
    run(transaction, "DELETE FROM document_shared_files WHERE document_id = $1", &[document_id]).await
}

async fn move_folder(
    transaction: &mut Transaction<'static, Any>,
    folder_id: &str,
    tree: &[String],
    previous_owner_id: &str,
    new_owner_id: &str,
) -> Result<(), ApiError> {
    for tree_folder_id in tree {
        let binds = [new_owner_id, tree_folder_id.as_str(), previous_owner_id];
        run(transaction, "UPDATE documents SET owner_id = $1 WHERE folder_id = $2 AND owner_id = $3", &binds).await?;
        run(transaction, "UPDATE projects SET owner_id = $1 WHERE folder_id = $2 AND owner_id = $3", &binds).await?;
        run(transaction, "UPDATE files SET owner_id = $1 WHERE folder_id = $2 AND owner_id = $3", &binds).await?;
        run(transaction, "UPDATE folders SET owner_id = $1 WHERE id = $2 AND owner_id = $3", &binds).await?;
        run(transaction, "DELETE FROM folder_collaborators WHERE folder_id = $1 AND user_id = $2", &[tree_folder_id.as_str(), new_owner_id]).await?;
    }
    run(transaction, "UPDATE folders SET parent_id = NULL WHERE id = $1", &[folder_id]).await
}

pub async fn transfer_item(
    state: &AppState,
    kind: ShareKind,
    item_id: &str,
    previous_owner_id: &str,
    new_owner_id: &str,
) -> Result<(), ApiError> {
    let tree = match kind {
        ShareKind::Folder => folder_tree(state, item_id, previous_owner_id).await?,
        _ => Vec::new(),
    };
    let mut transaction = state.db.begin().await.map_err(internal_error)?;

    let claimed = sqlx::query(
        sql("DELETE FROM ownership_transfers WHERE item_kind = $1 AND item_id = $2 AND from_user_id = $3 AND to_user_id = $4")
    )
    .bind(kind.label())
    .bind(item_id)
    .bind(previous_owner_id)
    .bind(new_owner_id)
    .execute(&mut *transaction)
    .await
    .map_err(internal_error)?;
    if claimed.rows_affected() == 0 {
        return Err((StatusCode::CONFLICT, "This transfer is no longer available".to_string()));
    }

    match kind {
        ShareKind::Document => move_document(&mut transaction, item_id, previous_owner_id, new_owner_id).await?,
        ShareKind::Project => {
            run(&mut transaction, "UPDATE projects SET owner_id = $1, folder_id = NULL WHERE id = $2", &[new_owner_id, item_id]).await?
        }
        ShareKind::Folder => move_folder(&mut transaction, item_id, &tree, previous_owner_id, new_owner_id).await?,
    }
    swap_membership(&mut transaction, kind, item_id, previous_owner_id, new_owner_id).await?;

    transaction.commit().await.map_err(internal_error)
}

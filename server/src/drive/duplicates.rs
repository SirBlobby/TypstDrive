use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Serialize;
use sqlx::{Any, Transaction};
use uuid::Uuid;

use crate::{
    db::sql,
    drive::{
        documents::saved_text,
        projects::{encode_text_blob, project_role},
    },
    realtime::live_documents::{document_room, live_text, project_file_room},
    sharing::access::{document_role, forbidden},
    AppState,
};

type ApiError = (StatusCode, String);

type ProjectFileRow = (String, String, String, Option<Vec<u8>>, String);

const MAX_NAME_LENGTH: usize = 200;

#[derive(Serialize)]
pub struct DuplicateCreated {
    pub id: String,
}

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn session_user(jar: &SignedCookieJar) -> Result<String, ApiError> {
    jar.get("session_user_id")
        .map(|cookie| cookie.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))
}

fn copy_name(name: &str) -> String {
    let suffix = " (copy)";
    let base: String = name.chars().take(MAX_NAME_LENGTH - suffix.len()).collect();
    format!("{base}{suffix}")
}

fn destination_folder(item_owner_id: &str, item_folder_id: &Option<String>, user_id: &str) -> Option<String> {
    if item_owner_id == user_id {
        item_folder_id.clone()
    } else {
        None
    }
}

async fn file_rows(state: &AppState, statement: &'static str, binds: &[&str]) -> Result<Vec<(String, i64)>, ApiError> {
    let mut query = sqlx::query_as::<_, (String, i64)>(sql(statement));
    for value in binds {
        query = query.bind(value.to_string());
    }
    query.fetch_all(&state.db).await.map_err(internal_error)
}

async fn copy_file(transaction: &mut Transaction<'static, Any>, file_id: &str, owner_id: &str, document_id: &str) -> Result<(), ApiError> {
    sqlx::query(sql(
        "INSERT INTO files (id, owner_id, document_id, folder_id, name, mime_type, data, size_bytes, storage_key) \
         SELECT $1, $2, $3, NULL, name, mime_type, data, size_bytes, storage_key FROM files WHERE id = $4",
    ))
    .bind(Uuid::new_v4().to_string())
    .bind(owner_id)
    .bind(document_id)
    .bind(file_id)
    .execute(&mut **transaction)
    .await
    .map_err(internal_error)?;
    Ok(())
}

async fn link_file(transaction: &mut Transaction<'static, Any>, file_id: &str, document_id: &str) -> Result<(), ApiError> {
    sqlx::query(sql("INSERT INTO document_shared_files (document_id, file_id) VALUES ($1, $2)"))
        .bind(document_id)
        .bind(file_id)
        .execute(&mut **transaction)
        .await
        .map_err(internal_error)?;
    Ok(())
}

pub async fn duplicate_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<DuplicateCreated>, ApiError> {
    let user_id = session_user(&jar)?;
    let (document, _) = document_role(&state, &id, &Some(user_id.clone())).await.ok_or_else(forbidden)?;
    let same_owner = document.owner_id == user_id;

    let text = match live_text(&state, &document_room(&id)).await {
        Some(text) => text,
        None => saved_text(&state, &id).await,
    };
    let content = encode_text_blob(&text);

    let attached_files = file_rows(&state, "SELECT id, size_bytes FROM files WHERE document_id = $1 AND deleted_at IS NULL", &[id.as_str()]).await?;
    let linked_files = file_rows(
        &state,
        "SELECT f.id, f.size_bytes FROM files f INNER JOIN document_shared_files l ON l.file_id = f.id \
         WHERE l.document_id = $1 AND f.deleted_at IS NULL AND (f.document_id IS NULL OR f.document_id != $2)",
        &[id.as_str(), id.as_str()],
    )
    .await?;

    let copied_file_bytes: i64 = attached_files
        .iter()
        .chain(linked_files.iter().filter(|_| !same_owner))
        .map(|(_, size)| size)
        .sum();
    crate::accounts::storage::ensure_room(&state, &user_id, content.len() as i64 + copied_file_bytes).await?;

    let copy_id = Uuid::new_v4().to_string();
    let folder_id = destination_folder(&document.owner_id, &document.folder_id, &user_id);
    let mut transaction = state.db.begin().await.map_err(internal_error)?;

    sqlx::query(sql(
        "INSERT INTO documents (id, owner_id, folder_id, title, content, language) \
         SELECT $1, $2, $3, $4, $5, language FROM documents WHERE id = $6",
    ))
    .bind(&copy_id)
    .bind(&user_id)
    .bind(&folder_id)
    .bind(copy_name(&document.title))
    .bind(&content)
    .bind(&id)
    .execute(&mut *transaction)
    .await
    .map_err(internal_error)?;

    for (file_id, _) in &attached_files {
        copy_file(&mut transaction, file_id, &user_id, &copy_id).await?;
    }
    for (file_id, _) in &linked_files {
        if same_owner {
            link_file(&mut transaction, file_id, &copy_id).await?;
        } else {
            copy_file(&mut transaction, file_id, &user_id, &copy_id).await?;
        }
    }

    transaction.commit().await.map_err(internal_error)?;
    Ok(Json(DuplicateCreated { id: copy_id }))
}

async fn current_file_content(state: &AppState, project_id: &str, file_id: &str, kind: &str, stored: Option<Vec<u8>>) -> Vec<u8> {
    if kind == "text" {
        if let Some(text) = live_text(state, &project_file_room(project_id, file_id)).await {
            return encode_text_blob(&text);
        }
    }
    stored.unwrap_or_default()
}

pub async fn duplicate_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<DuplicateCreated>, ApiError> {
    let user_id = session_user(&jar)?;
    let (project, _) = project_role(&state, &id, &Some(user_id.clone())).await.ok_or_else(forbidden)?;

    let rows = sqlx::query_as::<_, ProjectFileRow>(sql("SELECT id, path, kind, content, mime_type FROM project_files WHERE project_id = $1"))
        .bind(&id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;

    let mut files = Vec::with_capacity(rows.len());
    for (file_id, path, kind, stored, mime_type) in rows {
        let content = current_file_content(&state, &id, &file_id, &kind, stored).await;
        files.push((path, kind, content, mime_type));
    }
    let total_bytes: i64 = files.iter().map(|(_, _, content, _)| content.len() as i64).sum();
    crate::accounts::storage::ensure_room(&state, &user_id, total_bytes).await?;

    let copy_id = Uuid::new_v4().to_string();
    let folder_id = destination_folder(&project.owner_id, &project.folder_id, &user_id);
    let mut transaction = state.db.begin().await.map_err(internal_error)?;

    sqlx::query(sql("INSERT INTO projects (id, owner_id, folder_id, name, entrypoint) VALUES ($1, $2, $3, $4, $5)"))
        .bind(&copy_id)
        .bind(&user_id)
        .bind(&folder_id)
        .bind(copy_name(&project.name))
        .bind(&project.entrypoint)
        .execute(&mut *transaction)
        .await
        .map_err(internal_error)?;

    for (path, kind, content, mime_type) in files {
        sqlx::query(sql("INSERT INTO project_files (id, project_id, path, kind, content, mime_type) VALUES ($1, $2, $3, $4, $5, $6)"))
            .bind(Uuid::new_v4().to_string())
            .bind(&copy_id)
            .bind(path)
            .bind(kind)
            .bind(content)
            .bind(mime_type)
            .execute(&mut *transaction)
            .await
            .map_err(internal_error)?;
    }

    transaction.commit().await.map_err(internal_error)?;
    Ok(Json(DuplicateCreated { id: copy_id }))
}

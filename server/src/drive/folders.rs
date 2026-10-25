use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    models::{Folder, CreateFolderRequest},
    AppState,
};
use crate::db::sql;

#[derive(Deserialize)]
pub struct ListFoldersQuery {
    pub parent_id: Option<String>,
}

pub async fn list_folders(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Query(query): Query<ListFoldersQuery>,
) -> Result<Json<Vec<Folder>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let folders = if let Some(parent_id) = query.parent_id {
        crate::sharing::access::require_folder_access(&state, &parent_id, &user_id).await?;
        sqlx::query_as::<_, Folder>(
            sql("SELECT id, owner_id, parent_id, name, created_at, color FROM folders WHERE parent_id = $1 AND owner_id = (SELECT p.owner_id FROM folders p WHERE p.id = $2) AND deleted_at IS NULL ORDER BY name ASC")
        )
        .bind(&parent_id)
        .bind(&parent_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        sqlx::query_as::<_, Folder>(
            sql("SELECT id, owner_id, parent_id, name, created_at, color FROM folders WHERE owner_id = $1 AND parent_id IS NULL AND deleted_at IS NULL ORDER BY name ASC")
        )
        .bind(&user_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    Ok(Json(folders))
}

pub async fn list_shared_folders(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<Folder>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let folders = sqlx::query_as::<_, Folder>(
        sql("SELECT f.id, f.owner_id, f.parent_id, f.name, f.created_at, f.color, c.role AS effective_role \
         FROM folders f INNER JOIN folder_collaborators c ON c.folder_id = f.id AND c.user_id = $1 \
         WHERE f.deleted_at IS NULL ORDER BY f.name ASC")
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(Json(folders))
}

pub async fn create_folder(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<CreateFolderRequest>,
) -> Result<Json<Folder>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;
    crate::sharing::access::require_owned_folder(&state, &payload.parent_id, &user_id).await?;

    let folder_id = Uuid::new_v4().to_string();

    sqlx::query(sql("INSERT INTO folders (id, owner_id, parent_id, name) VALUES ($1, $2, $3, $4)"))
    .bind(&folder_id)
    .bind(&user_id)
    .bind(&payload.parent_id)
    .bind(&payload.name)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let folder = sqlx::query_as::<_, Folder>(sql("SELECT id, owner_id, parent_id, name, created_at, color FROM folders WHERE id = $1"))
    .bind(&folder_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(folder))
}

pub async fn delete_folder(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    session: Option<axum::Extension<crate::accounts::sessions::ActiveSession>>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;
    let deleted_by = session.map(|axum::Extension(active)| active.user_id).unwrap_or_else(|| user_id.clone());
    crate::trash::move_to_trash(&state, crate::trash::TrashKind::Folder, &id, &user_id, &deleted_by).await?;

    Ok(StatusCode::NO_CONTENT)
}

fn internal_error(error: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

pub(crate) async fn items_in_folder_tree(
    state: &AppState,
    table: &str,
    folder_id: &str,
    user_id: &str,
) -> Result<Vec<String>, (StatusCode, String)> {
    let query = if table == "projects" {
        "WITH RECURSIVE tree(id) AS ( \
             SELECT id FROM folders WHERE id = $1 AND owner_id = $2 \
             UNION ALL SELECT f.id FROM folders f INNER JOIN tree ON f.parent_id = tree.id \
         ) SELECT id FROM projects WHERE folder_id IN (SELECT id FROM tree)"
    } else {
        "WITH RECURSIVE tree(id) AS ( \
             SELECT id FROM folders WHERE id = $1 AND owner_id = $2 \
             UNION ALL SELECT f.id FROM folders f INNER JOIN tree ON f.parent_id = tree.id \
         ) SELECT id FROM documents WHERE folder_id IN (SELECT id FROM tree)"
    };

    let rows = sqlx::query_as::<_, (String,)>(sql(query))
        .bind(folder_id)
        .bind(user_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

#[derive(Deserialize)]
pub struct UpdateFolderRequest {
    pub name: String,
}

pub async fn update_folder(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    Json(payload): Json<UpdateFolderRequest>,
) -> Result<Json<Folder>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    sqlx::query(sql("UPDATE folders SET name = $1 WHERE id = $2 AND owner_id = $3"))
    .bind(&payload.name)
    .bind(&id)
    .bind(&user_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let folder = sqlx::query_as::<_, Folder>(sql("SELECT id, owner_id, parent_id, name, created_at, color FROM folders WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL"))
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match folder {
        Some(f) => Ok(Json(f)),
        None => Err((StatusCode::NOT_FOUND, "Folder not found".to_string())),
    }
}

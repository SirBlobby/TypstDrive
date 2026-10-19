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
        sqlx::query_as::<_, Folder>(
            "SELECT id, owner_id, parent_id, name, created_at FROM folders WHERE owner_id = $1 AND parent_id = $2 ORDER BY name ASC"
        )
        .bind(&user_id)
        .bind(&parent_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        sqlx::query_as::<_, Folder>(
            "SELECT id, owner_id, parent_id, name, created_at FROM folders WHERE owner_id = $1 AND parent_id IS NULL ORDER BY name ASC"
        )
        .bind(&user_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    Ok(Json(folders))
}

pub async fn create_folder(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<CreateFolderRequest>,
) -> Result<Json<Folder>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let folder_id = Uuid::new_v4().to_string();

    let folder = sqlx::query_as::<_, Folder>(
        "INSERT INTO folders (id, owner_id, parent_id, name) VALUES ($1, $2, $3, $4) RETURNING id, owner_id, parent_id, name, created_at"
    )
    .bind(&folder_id)
    .bind(&user_id)
    .bind(&payload.parent_id)
    .bind(&payload.name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(folder))
}

pub async fn delete_folder(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let owned_folder = sqlx::query_as::<_, (String,)>("SELECT id FROM folders WHERE id = $1 AND owner_id = $2")
        .bind(&id)
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;

    if owned_folder.is_none() {
        return Err((StatusCode::NOT_FOUND, "Folder not found or unauthorized".to_string()));
    }

    let affected_documents = items_in_folder_tree(&state, "documents", &id, &user_id).await?;
    let affected_projects = items_in_folder_tree(&state, "projects", &id, &user_id).await?;

    sqlx::query("DELETE FROM folders WHERE id = $1 AND owner_id = $2")
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    for document_id in affected_documents {
        crate::access::revalidate_resource(&state, crate::access::Resource::Document(document_id)).await;
    }
    for project_id in affected_projects {
        crate::access::revalidate_resource(&state, crate::access::Resource::Project(project_id)).await;
    }

    Ok(StatusCode::NO_CONTENT)
}

fn internal_error(error: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

async fn items_in_folder_tree(
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

    let rows = sqlx::query_as::<_, (String,)>(query)
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

    let folder = sqlx::query_as::<_, Folder>(
        "UPDATE folders SET name = $1 WHERE id = $2 AND owner_id = $3 RETURNING id, owner_id, parent_id, name, created_at"
    )
    .bind(&payload.name)
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

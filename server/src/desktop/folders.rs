use super::*;
use crate::db::sql;

#[derive(Serialize)]
pub struct CloudFolder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

pub async fn list_folders(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<CloudFolder>>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
        sql("SELECT id, name, parent_id FROM folders WHERE owner_id = $1 ORDER BY name ASC"),
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, name, parent_id)| CloudFolder {
                id,
                name,
                parent_id,
            })
            .collect(),
    ))
}

#[derive(Deserialize)]
pub struct CreateFolderBody {
    pub name: String,
    pub parent_id: Option<String>,
}

pub async fn create_folder(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateFolderBody>,
) -> Result<Json<CloudFolder>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }

    if let Some(parent_id) = &payload.parent_id {
        owned_folder(&state, parent_id, &user_id).await?;
    }

    let folder_id = Uuid::new_v4().to_string();
    let name = payload.name.trim().to_string();

    sqlx::query(sql("INSERT INTO folders (id, owner_id, parent_id, name) VALUES ($1, $2, $3, $4)"))
        .bind(&folder_id)
        .bind(&user_id)
        .bind(&payload.parent_id)
        .bind(&name)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(CloudFolder {
        id: folder_id,
        name,
        parent_id: payload.parent_id,
    }))
}

#[derive(Deserialize)]
pub struct RenameFolderBody {
    pub name: String,
}

pub async fn rename_folder(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<RenameFolderBody>,
) -> Result<Json<CloudFolder>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }

    sqlx::query(sql("UPDATE folders SET name = $1 WHERE id = $2 AND owner_id = $3"))
    .bind(payload.name.trim())
    .bind(&id)
    .bind(&user_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let row = sqlx::query_as::<_, (String, Option<String>)>(sql("SELECT name, parent_id FROM folders WHERE id = $1 AND owner_id = $2"))
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Folder not found".to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(CloudFolder {
        id,
        name: row.0,
        parent_id: row.1,
    }))
}

async fn creates_cycle(
    state: &AppState,
    folder_id: &str,
    new_parent_id: &str,
    user_id: &str,
) -> Result<bool, (StatusCode, String)> {
    let mut current = new_parent_id.to_string();

    for _ in 0..1000 {
        if current == folder_id {
            return Ok(true);
        }

        let parent = sqlx::query_as::<_, (Option<String>,)>(
            sql("SELECT parent_id FROM folders WHERE id = $1 AND owner_id = $2"),
        )
        .bind(&current)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .and_then(|row| row.0);

        match parent {
            Some(next) => current = next,
            None => return Ok(false),
        }
    }

    Ok(true)
}

#[derive(Deserialize)]
pub struct MoveFolderBody {
    pub parent_id: Option<String>,
}

pub async fn move_folder(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<MoveFolderBody>,
) -> Result<Json<CloudFolder>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    owned_folder(&state, &id, &user_id).await?;

    if let Some(parent_id) = &payload.parent_id {
        if parent_id == &id {
            return Err((
                StatusCode::BAD_REQUEST,
                "A folder cannot be moved into itself".to_string(),
            ));
        }

        owned_folder(&state, parent_id, &user_id).await?;

        if creates_cycle(&state, &id, parent_id, &user_id).await? {
            return Err((
                StatusCode::BAD_REQUEST,
                "Cannot move a folder into one of its own subfolders".to_string(),
            ));
        }
    }

    sqlx::query(sql("UPDATE folders SET parent_id = $1 WHERE id = $2 AND owner_id = $3"))
    .bind(&payload.parent_id)
    .bind(&id)
    .bind(&user_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let row = sqlx::query_as::<_, (String, Option<String>)>(sql("SELECT name, parent_id FROM folders WHERE id = $1 AND owner_id = $2"))
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Folder not found".to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(CloudFolder {
        id,
        name: row.0,
        parent_id: row.1,
    }))
}

async fn folder_is_empty(
    state: &AppState,
    folder_id: &str,
    user_id: &str,
) -> Result<bool, (StatusCode, String)> {
    let row = sqlx::query_as::<_, (i64, i64, i64)>(
        sql("SELECT \
           (SELECT COUNT(*) FROM folders WHERE parent_id = $1 AND owner_id = $2), \
           (SELECT COUNT(*) FROM projects WHERE folder_id = $3 AND owner_id = $4), \
           (SELECT COUNT(*) FROM documents WHERE folder_id = $5 AND owner_id = $6)"),
    )
    .bind(folder_id)
    .bind(user_id)
    .bind(folder_id)
    .bind(user_id)
    .bind(folder_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(row.0 == 0 && row.1 == 0 && row.2 == 0)
}

pub async fn delete_folder(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    owned_folder(&state, &id, &user_id).await?;

    if !folder_is_empty(&state, &id, &user_id).await? {
        return Err((
            StatusCode::BAD_REQUEST,
            "Move or delete everything inside this folder first".to_string(),
        ));
    }

    let result = sqlx::query(sql("DELETE FROM folders WHERE id = $1 AND owner_id = $2"))
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Folder not found or unauthorized".to_string()));
    }

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(StatusCode::NO_CONTENT)
}

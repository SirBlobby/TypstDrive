use super::*;
use super::documents::FolderQuery;
use crate::db::sql;

#[derive(Serialize)]
pub struct CloudFile {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub folder_id: Option<String>,
    pub created_at: String,
}

pub async fn list_account_files(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<FolderQuery>,
) -> Result<Json<Vec<CloudFile>>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let rows = match &query.folder_id {
        Some(folder_id) => sqlx::query_as::<_, (String, String, String, Option<String>, String)>(
            sql("SELECT id, name, mime_type, folder_id, created_at FROM files \
             WHERE owner_id = $1 AND folder_id = $2 ORDER BY name ASC"),
        )
        .bind(&user_id)
        .bind(folder_id)
        .fetch_all(&state.db)
        .await,
        None => sqlx::query_as::<_, (String, String, String, Option<String>, String)>(
            sql("SELECT id, name, mime_type, folder_id, created_at FROM files \
             WHERE owner_id = $1 AND folder_id IS NULL ORDER BY name ASC"),
        )
        .bind(&user_id)
        .fetch_all(&state.db)
        .await,
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, name, mime_type, folder_id, created_at)| CloudFile {
                id,
                name,
                mime_type,
                folder_id,
                created_at,
            })
            .collect(),
    ))
}

#[derive(Deserialize)]
pub struct UploadFileBody {
    pub name: String,
    pub mime_type: String,
    pub encoding: String,
    pub content: String,
    pub folder_id: Option<String>,
}

pub async fn upload_account_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UploadFileBody>,
) -> Result<Json<CloudFile>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }

    if let Some(folder_id) = &payload.folder_id {
        owned_folder(&state, folder_id, &user_id).await?;
    }

    let data = match payload.encoding.as_str() {
        "base64" => BASE64
            .decode(payload.content.as_bytes())
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid base64: {}", e)))?,
        _ => payload.content.into_bytes(),
    };

    crate::accounts::storage::ensure_room(&state, &user_id, data.len() as i64).await?;

    let file_id = Uuid::new_v4().to_string();
    let name = payload.name.trim().to_string();

    let stored = state.file_storage.store(data).await?;
    sqlx::query(sql("INSERT INTO files (id, owner_id, folder_id, name, mime_type, data, size_bytes, storage_key) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"))
    .bind(&file_id)
    .bind(&user_id)
    .bind(&payload.folder_id)
    .bind(&name)
    .bind(&payload.mime_type)
    .bind(&stored.inline_data)
    .bind(stored.size_bytes)
    .bind(&stored.storage_key)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let row = sqlx::query_as::<_, (String,)>(sql("SELECT created_at FROM files WHERE id = $1"))
    .bind(&file_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(CloudFile {
        id: file_id,
        name,
        mime_type: payload.mime_type,
        folder_id: payload.folder_id,
        created_at: row.0,
    }))
}

#[derive(Deserialize)]
pub struct RenameFileBody {
    pub name: String,
}

pub async fn rename_account_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<RenameFileBody>,
) -> Result<Json<CloudFile>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }

    sqlx::query(sql("UPDATE files SET name = $1 WHERE id = $2 AND owner_id = $3"))
    .bind(payload.name.trim())
    .bind(&id)
    .bind(&user_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let row = sqlx::query_as::<_, (String, String, Option<String>, String)>(sql("SELECT name, mime_type, folder_id, created_at FROM files WHERE id = $1 AND owner_id = $2"))
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(CloudFile {
        id,
        name: row.0,
        mime_type: row.1,
        folder_id: row.2,
        created_at: row.3,
    }))
}

#[derive(Deserialize)]
pub struct MoveFileBody {
    pub folder_id: Option<String>,
}

pub async fn move_account_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<MoveFileBody>,
) -> Result<Json<CloudFile>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    if let Some(folder_id) = &payload.folder_id {
        owned_folder(&state, folder_id, &user_id).await?;
    }

    sqlx::query(sql("UPDATE files SET folder_id = $1 WHERE id = $2 AND owner_id = $3"))
    .bind(&payload.folder_id)
    .bind(&id)
    .bind(&user_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let row = sqlx::query_as::<_, (String, String, Option<String>, String)>(sql("SELECT name, mime_type, folder_id, created_at FROM files WHERE id = $1 AND owner_id = $2"))
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(CloudFile {
        id,
        name: row.0,
        mime_type: row.1,
        folder_id: row.2,
        created_at: row.3,
    }))
}

#[derive(Serialize)]
pub struct CloudFileContent {
    pub name: String,
    pub mime_type: String,
    pub encoding: String,
    pub content: String,
}

pub async fn pull_account_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<CloudFileContent>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let row = sqlx::query_as::<_, (String, String, Vec<u8>, Option<String>)>(
        sql("SELECT name, mime_type, data, storage_key FROM files WHERE id = $1 AND owner_id = $2"),
    )
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    let (name, mime_type, inline_data, storage_key) = row;
    let data = state.file_storage.load(inline_data, storage_key).await?;

    Ok(Json(CloudFileContent {
        name,
        mime_type,
        encoding: "base64".to_string(),
        content: BASE64.encode(&data),
    }))
}

pub async fn delete_account_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let result = sqlx::query(sql("DELETE FROM files WHERE id = $1 AND owner_id = $2"))
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "File not found or unauthorized".to_string()));
    }

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(StatusCode::NO_CONTENT)
}

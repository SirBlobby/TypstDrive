use super::*;
use crate::db::sql;

#[derive(Serialize)]
pub struct ManifestEntry {
    pub id: String,
    pub path: String,
    pub kind: String,
    pub hash: String,
    pub size: usize,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct ProjectManifest {
    pub project_id: String,
    pub name: String,
    pub entrypoint: String,
    pub updated_at: String,
    pub files: Vec<ManifestEntry>,
}

async fn plain_contents(
    state: &AppState,
    project_id: &str,
) -> Result<Vec<(String, String, String, Vec<u8>, String)>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, (String, String, String, Option<Vec<u8>>, Option<String>)>(
        sql("SELECT id, path, kind, content, updated_at FROM project_files WHERE project_id = $1 ORDER BY path ASC"),
    )
    .bind(project_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut contents = Vec::with_capacity(rows.len());
    for (id, path, kind, content, updated_at) in rows {
        let plain = plain_file_content(state, project_id, &id, &kind, content).await;
        contents.push((id, path, kind, plain, updated_at.unwrap_or_default()));
    }
    Ok(contents)
}

async fn plain_file_content(
    state: &AppState,
    project_id: &str,
    file_id: &str,
    kind: &str,
    stored: Option<Vec<u8>>,
) -> Vec<u8> {
    let raw = stored.unwrap_or_default();
    if kind == "binary" {
        return raw;
    }
    let room = crate::realtime::live_documents::project_file_room(project_id, file_id);
    match crate::realtime::live_documents::live_text(state, &room).await {
        Some(live) => live.into_bytes(),
        None => decode_text_blob(&raw).into_bytes(),
    }
}

pub async fn get_manifest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
) -> Result<Json<ProjectManifest>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    let project = readable_project(&state, &project_id, &user_id).await?;

    let files = plain_contents(&state, &project_id)
        .await?
        .into_iter()
        .map(|(id, path, kind, plain, updated_at)| ManifestEntry {
            id,
            path,
            kind,
            hash: content_hash(&plain),
            size: plain.len(),
            updated_at,
        })
        .collect();

    Ok(Json(ProjectManifest {
        project_id: project.id,
        name: project.name,
        entrypoint: project.entrypoint,
        updated_at: project.updated_at,
        files,
    }))
}

#[derive(Deserialize)]
pub struct PathQuery {
    pub path: String,
}

#[derive(Serialize)]
pub struct FileContent {
    pub path: String,
    pub kind: String,
    pub hash: String,
    pub encoding: String,
    pub content: String,
}

fn encode_for_transport(kind: &str, plain: Vec<u8>) -> (String, String) {
    if kind == "binary" {
        ("base64".to_string(), BASE64.encode(&plain))
    } else {
        (
            "utf8".to_string(),
            String::from_utf8_lossy(&plain).to_string(),
        )
    }
}

pub async fn pull_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    Query(query): Query<PathQuery>,
) -> Result<Json<FileContent>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    readable_project(&state, &project_id, &user_id).await?;

    let row = sqlx::query_as::<_, (String, String, Option<Vec<u8>>)>(
        sql("SELECT id, kind, content FROM project_files WHERE project_id = $1 AND path = $2"),
    )
    .bind(&project_id)
    .bind(&query.path)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    let (file_id, kind, content) = row;
    let plain = plain_file_content(&state, &project_id, &file_id, &kind, content).await;

    let hash = content_hash(&plain);
    let (encoding, content) = encode_for_transport(&kind, plain);

    Ok(Json(FileContent {
        path: query.path,
        kind,
        hash,
        encoding,
        content,
    }))
}

#[derive(Deserialize)]
pub struct PushFileRequest {
    pub path: String,
    pub content: String,
    pub encoding: Option<String>,
    pub base_hash: Option<String>,
}

#[derive(Serialize)]
pub struct PushFileResponse {
    pub path: String,
    pub hash: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct ConflictResponse {
    pub conflict: bool,
    pub path: String,
    pub server_hash: String,
    pub base_hash: Option<String>,
    pub encoding: String,
    pub server_content: String,
}

pub enum PushOutcome {
    Applied(Json<PushFileResponse>),
    Conflict(Json<ConflictResponse>),
}

impl axum::response::IntoResponse for PushOutcome {
    fn into_response(self) -> axum::response::Response {
        match self {
            PushOutcome::Applied(body) => (StatusCode::OK, body).into_response(),
            PushOutcome::Conflict(body) => (StatusCode::CONFLICT, body).into_response(),
        }
    }
}

pub async fn push_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    Json(payload): Json<PushFileRequest>,
) -> Result<PushOutcome, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    let project = writable_project(&state, &project_id, &user_id).await?;

    let incoming = match payload.encoding.as_deref() {
        Some("base64") => BASE64
            .decode(payload.content.as_bytes())
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid base64: {}", e)))?,
        _ => payload.content.clone().into_bytes(),
    };

    let existing = sqlx::query_as::<_, (String, String, Option<Vec<u8>>)>(
        sql("SELECT id, kind, content FROM project_files WHERE project_id = $1 AND path = $2"),
    )
    .bind(&project_id)
    .bind(&payload.path)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some((existing_id, existing_kind, existing_content)) = &existing {
        let plain = plain_file_content(&state, &project_id, existing_id, existing_kind, existing_content.clone()).await;
        let server_hash = content_hash(&plain);

        let safe = match &payload.base_hash {
            Some(base) => base == &server_hash,
            None => false,
        };

        if !safe && server_hash != content_hash(&incoming) {
            let (encoding, server_content) = encode_for_transport(existing_kind, plain);
            return Ok(PushOutcome::Conflict(Json(ConflictResponse {
                conflict: true,
                path: payload.path,
                server_hash,
                base_hash: payload.base_hash,
                encoding,
                server_content,
            })));
        }
    }

    let kind = if payload.encoding.as_deref() == Some("base64") && !is_text_path(&payload.path) {
        "binary"
    } else {
        "text"
    };

    let stored = if kind == "binary" {
        incoming.clone()
    } else {
        encode_text_blob(&String::from_utf8_lossy(&incoming))
    };

    let mime_type = if kind == "binary" {
        "application/octet-stream"
    } else {
        "text/plain"
    };

    let existing_size = existing
        .as_ref()
        .and_then(|(_, _, content)| content.as_ref())
        .map(|content| content.len() as i64)
        .unwrap_or(0);
    crate::accounts::storage::ensure_room(&state, &project.owner_id, stored.len() as i64 - existing_size).await?;

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    sqlx::query(
        sql("INSERT INTO project_files (id, project_id, path, kind, content, mime_type, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         ON CONFLICT (project_id, path) DO UPDATE SET content = excluded.content, \
         kind = excluded.kind, mime_type = excluded.mime_type, updated_at = excluded.updated_at"),
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&project_id)
    .bind(&payload.path)
    .bind(kind)
    .bind(&stored)
    .bind(mime_type)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = sqlx::query(sql("UPDATE projects SET updated_at = $1 WHERE id = $2"))
        .bind(&now)
        .bind(&project_id)
        .execute(&state.db)
        .await;

    if kind == "text" {
        let file_id = sqlx::query_as::<_, (String,)>(sql("SELECT id FROM project_files WHERE project_id = $1 AND path = $2"))
            .bind(&project_id)
            .bind(&payload.path)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();
        if let Some((file_id,)) = file_id {
            let room = crate::realtime::live_documents::project_file_room(&project_id, &file_id);
            crate::realtime::live_documents::replace_text(&state, &room, &String::from_utf8_lossy(&incoming)).await;
        }
    }

    notify_devices(&state, &project.owner_id, DeviceEvent::project(&project_id)).await;

    Ok(PushOutcome::Applied(Json(PushFileResponse {
        path: payload.path,
        hash: content_hash(&incoming),
        updated_at: now,
    })))
}

pub async fn delete_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    Query(query): Query<PathQuery>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    writable_project(&state, &project_id, &user_id).await?;

    let result = sqlx::query(sql("DELETE FROM project_files WHERE project_id = $1 AND path = $2"))
        .bind(&project_id)
        .bind(&query.path)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    notify_devices(&state, &user_id, DeviceEvent::project(&project_id)).await;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct BundleFile {
    pub path: String,
    pub kind: String,
    pub hash: String,
    pub encoding: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ProjectBundle {
    pub project_id: String,
    pub name: String,
    pub entrypoint: String,
    pub files: Vec<BundleFile>,
}

pub async fn pull_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
) -> Result<Json<ProjectBundle>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    let project = readable_project(&state, &project_id, &user_id).await?;

    let files = plain_contents(&state, &project_id)
        .await?
        .into_iter()
        .map(|(_, path, kind, plain, _)| {
            let hash = content_hash(&plain);
            let (encoding, content) = encode_for_transport(&kind, plain);
            BundleFile {
                path,
                kind,
                hash,
                encoding,
                content,
            }
        })
        .collect();

    Ok(Json(ProjectBundle {
        project_id: project.id,
        name: project.name,
        entrypoint: project.entrypoint,
        files,
    }))
}

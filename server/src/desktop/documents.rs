use super::*;
use super::project_files::{ConflictResponse, PushFileResponse, PushOutcome};
use super::projects::ProjectSummary;
use crate::db::sql;

#[derive(Serialize)]
pub struct CloudDocument {
    pub id: String,
    pub title: String,
    pub folder_id: Option<String>,
    pub role: String,
    pub updated_at: String,
    pub thumbnail_svg: Option<String>,
}

#[derive(Deserialize)]
pub struct FolderQuery {
    pub folder_id: Option<String>,
}

pub async fn list_documents(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<FolderQuery>,
) -> Result<Json<Vec<CloudDocument>>, (StatusCode, String)> {
    let user_id = readable_space(&state, &headers).await?;

    let rows = match &query.folder_id {
        Some(folder_id) => sqlx::query_as::<_, (String, String, Option<String>, String, Option<String>)>(
            sql("SELECT id, title, folder_id, updated_at, thumbnail_svg FROM documents \
             WHERE owner_id = $1 AND folder_id = $2 ORDER BY updated_at DESC"),
        )
        .bind(&user_id)
        .bind(folder_id)
        .fetch_all(&state.db)
        .await,
        None => sqlx::query_as::<_, (String, String, Option<String>, String, Option<String>)>(
            sql("SELECT id, title, folder_id, updated_at, thumbnail_svg FROM documents \
             WHERE owner_id = $1 AND folder_id IS NULL ORDER BY updated_at DESC"),
        )
        .bind(&user_id)
        .fetch_all(&state.db)
        .await,
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, title, folder_id, updated_at, thumbnail_svg)| CloudDocument {
                id,
                title,
                folder_id,
                role: "owner".to_string(),
                updated_at,
                thumbnail_svg,
            })
            .collect(),
    ))
}

#[derive(Serialize)]
pub struct SharedItems {
    pub documents: Vec<CloudDocument>,
    pub projects: Vec<ProjectSummary>,
}

pub async fn list_shared(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<SharedItems>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;

    let documents = sqlx::query_as::<_, (String, String, Option<String>, String, String, Option<String>)>(
        sql("SELECT d.id, d.title, d.folder_id, d.updated_at, c.role, d.thumbnail_svg FROM documents d \
         INNER JOIN collaborators c ON c.document_id = d.id AND c.user_id = $1 \
         ORDER BY d.updated_at DESC"),
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let projects = sqlx::query_as::<_, (String, String, String, String, String)>(
        sql("SELECT p.id, p.name, p.entrypoint, p.updated_at, c.role FROM projects p \
         INNER JOIN project_collaborators c ON c.project_id = p.id AND c.user_id = $1 \
         ORDER BY p.updated_at DESC"),
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(SharedItems {
        documents: documents
            .into_iter()
            .map(|(id, title, folder_id, updated_at, role, thumbnail_svg)| CloudDocument {
                id,
                title,
                folder_id,
                role,
                updated_at,
                thumbnail_svg,
            })
            .collect(),
        projects: projects
            .into_iter()
            .map(|(id, name, entrypoint, updated_at, role)| ProjectSummary {
                id,
                name,
                entrypoint,
                folder_id: None,
                role,
                updated_at,
            })
            .collect(),
    }))
}

async fn document_role(
    state: &AppState,
    document_id: &str,
    user_id: &str,
) -> Result<(String, String, String), (StatusCode, String)> {
    let row = sqlx::query_as::<_, (String, String, Option<Vec<u8>>, Option<String>, Option<String>, Option<String>)>(
        sql("SELECT owner_id, title, content, public_role, link_password_hash, link_expires_at FROM documents WHERE id = $1"),
    )
    .bind(document_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;

    let (owner_id, title, content, public_role, link_password_hash, link_expires_at) = row;

    let mut role = if owner_id == user_id {
        "owner".to_string()
    } else {
        "none".to_string()
    };

    if role == "none" {
        if let Ok(Some((collaborator_role,))) = sqlx::query_as::<_, (String,)>(
            sql("SELECT role FROM collaborators WHERE document_id = $1 AND user_id = $2"),
        )
        .bind(document_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        {
            role = collaborator_role;
        }
    }

    if role == "none" {
        if let Some(team_role) = crate::teams::content_role(state, &owner_id, user_id).await {
            role = if team_role.can_edit() { "editor".to_string() } else { "viewer".to_string() };
        }
    }

    if role == "none" {
        let link_role = crate::sharing::links::public_access(
            document_id,
            public_role.as_deref(),
            link_password_hash.as_deref(),
            link_expires_at.as_deref(),
        );
        if let Some(link_role) = link_role {
            role = link_role.as_str().to_string();
        }
    }

    if role == "none" {
        return Err((StatusCode::FORBIDDEN, "No access to this document".to_string()));
    }

    let text = decode_text_blob(&content.unwrap_or_default());
    Ok((role, title, text))
}

#[derive(Serialize)]
pub struct DocumentContent {
    pub id: String,
    pub title: String,
    pub role: String,
    pub hash: String,
    pub content: String,
    pub language: String,
}

async fn document_language(state: &AppState, document_id: &str) -> String {
    sqlx::query_as::<_, (Option<String>,)>(sql("SELECT language FROM documents WHERE id = $1"))
        .bind(document_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .and_then(|(language,)| language)
        .unwrap_or_else(|| crate::compile::latex::TYPST_LANGUAGE.to_string())
}

pub async fn pull_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<DocumentContent>, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    let (role, title, stored_content) = document_role(&state, &id, &user_id).await?;
    let content = crate::realtime::live_documents::live_text(&state, &crate::realtime::live_documents::document_room(&id))
        .await
        .unwrap_or(stored_content);
    let language = document_language(&state, &id).await;

    Ok(Json(DocumentContent {
        id,
        title,
        role,
        hash: content_hash(content.as_bytes()),
        content,
        language,
    }))
}

#[derive(Deserialize)]
pub struct PushDocumentRequest {
    pub content: String,
    pub base_hash: Option<String>,
}

pub async fn push_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<PushDocumentRequest>,
) -> Result<PushOutcome, (StatusCode, String)> {
    let user_id = authenticate(&state, &headers).await?;
    let (role, _, stored_content) = document_role(&state, &id, &user_id).await?;

    if role != "owner" && role != "editor" {
        return Err((StatusCode::FORBIDDEN, "Read-only access".to_string()));
    }

    let room = crate::realtime::live_documents::document_room(&id);
    let current = crate::realtime::live_documents::live_text(&state, &room).await.unwrap_or(stored_content);

    let server_hash = content_hash(current.as_bytes());
    let incoming_hash = content_hash(payload.content.as_bytes());

    let safe = match &payload.base_hash {
        Some(base) => base == &server_hash,
        None => false,
    };

    if !safe && server_hash != incoming_hash {
        return Ok(PushOutcome::Conflict(Json(ConflictResponse {
            conflict: true,
            path: id,
            server_hash,
            base_hash: payload.base_hash,
            encoding: "utf8".to_string(),
            server_content: current,
        })));
    }

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let stored_blob = sqlx::query_as::<_, (Option<Vec<u8>>,)>(sql("SELECT content FROM documents WHERE id = $1"))
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .and_then(|(content,)| content);
    let merged_blob = crate::realtime::live_documents::store_text(&state, &room, stored_blob.as_deref(), &payload.content).await;

    sqlx::query(sql("UPDATE documents SET content = $1, updated_at = $2 WHERE id = $3"))
        .bind(&merged_blob)
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::drive::activity::mark_document_edited(&state, &id, Some(&user_id)).await;
    let owner_id = document_owner(&state, &id).await.unwrap_or(user_id);
    notify_devices(&state, &owner_id, DeviceEvent::document(&id)).await;

    Ok(PushOutcome::Applied(Json(PushFileResponse {
        path: id,
        hash: incoming_hash,
        updated_at: now,
    })))
}

#[derive(Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub content: String,
    pub folder_id: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
}

pub async fn create_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateDocumentRequest>,
) -> Result<Json<DocumentContent>, (StatusCode, String)> {
    let user_id = writable_space(&state, &headers).await?;
    if let Some(folder_id) = &payload.folder_id {
        owned_folder(&state, folder_id, &user_id).await?;
    }
    crate::accounts::storage::ensure_room(&state, &user_id, payload.content.len() as i64).await?;
    let document_id = Uuid::new_v4().to_string();
    let language = crate::compile::latex::parse_language(payload.language.as_deref())?;

    sqlx::query(
        sql("INSERT INTO documents (id, owner_id, folder_id, title, content, language) VALUES ($1, $2, $3, $4, $5, $6)"),
    )
    .bind(&document_id)
    .bind(&user_id)
    .bind(&payload.folder_id)
    .bind(&payload.title)
    .bind(encode_text_blob(&payload.content))
    .bind(language)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(DocumentContent {
        id: document_id,
        title: payload.title,
        role: "owner".to_string(),
        hash: content_hash(payload.content.as_bytes()),
        content: payload.content,
        language: language.to_string(),
    }))
}

#[derive(Deserialize)]
pub struct MoveDocumentBody {
    pub folder_id: Option<String>,
}

pub async fn move_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<MoveDocumentBody>,
) -> Result<Json<CloudDocument>, (StatusCode, String)> {
    let user_id = writable_space(&state, &headers).await?;

    if let Some(folder_id) = &payload.folder_id {
        owned_folder(&state, folder_id, &user_id).await?;
    }

    sqlx::query(sql("UPDATE documents SET folder_id = $1 WHERE id = $2 AND owner_id = $3"))
    .bind(&payload.folder_id)
    .bind(&id)
    .bind(&user_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let row = sqlx::query_as::<_, (String, String, Option<String>, String, Option<String>)>(sql("SELECT id, title, folder_id, updated_at, thumbnail_svg FROM documents WHERE id = $1 AND owner_id = $2"))
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(CloudDocument {
        id: row.0,
        title: row.1,
        folder_id: row.2,
        role: "owner".to_string(),
        updated_at: row.3,
        thumbnail_svg: row.4,
    }))
}

pub async fn delete_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = writable_space(&state, &headers).await?;

    let result = sqlx::query(sql("DELETE FROM documents WHERE id = $1 AND owner_id = $2"))
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Document not found or unauthorized".to_string()));
    }

    crate::sharing::access::revalidate_resource(&state, crate::sharing::access::Resource::Document(id.clone())).await;
    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(StatusCode::NO_CONTENT)
}

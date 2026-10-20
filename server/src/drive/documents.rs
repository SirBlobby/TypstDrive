use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use uuid::Uuid;

use crate::{
    drive::projects::{decode_text_blob, encode_text_blob},
    models::{Document, CreateDocumentRequest},
    AppState,
};
use crate::db::sql;

const DOCUMENT_BY_ID: &str = "SELECT id, owner_id, folder_id, title, thumbnail_svg, public_role, created_at, updated_at FROM documents WHERE id = $1";
const OWNED_DOCUMENT: &str = "SELECT id, owner_id, folder_id, title, thumbnail_svg, public_role, created_at, updated_at FROM documents WHERE id = $1 AND owner_id = $2";

pub async fn stored_content(state: &AppState, document_id: &str) -> Option<Vec<u8>> {
    sqlx::query_as::<_, (Option<Vec<u8>>,)>(sql("SELECT content FROM documents WHERE id = $1"))
        .bind(document_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .and_then(|(content,)| content)
}

pub async fn saved_text(state: &AppState, document_id: &str) -> String {
    decode_text_blob(&stored_content(state, document_id).await.unwrap_or_default())
}

#[derive(serde::Deserialize)]
pub struct ListDocsQuery {
    pub folder_id: Option<String>,
}

pub async fn list_shared_documents(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<Document>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let docs = sqlx::query_as::<_, Document>(
        sql("SELECT d.id, d.owner_id, d.folder_id, d.title, d.thumbnail_svg, \
         d.public_role, d.created_at, d.updated_at, c.role as effective_role \
         FROM documents d \
         INNER JOIN collaborators c ON c.document_id = d.id AND c.user_id = $1 \
         ORDER BY d.updated_at DESC")
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(docs))
}

pub async fn list_documents(
    axum::extract::Query(query): axum::extract::Query<ListDocsQuery>,
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<Document>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let docs = if let Some(folder_id) = query.folder_id {
        crate::sharing::access::require_folder_access(&state, &folder_id, &user_id).await?;
        sqlx::query_as::<_, Document>(
            sql("SELECT id, owner_id, folder_id, title, thumbnail_svg, public_role, created_at, updated_at, CAST(COALESCE(LENGTH(content), 0) AS BIGINT) AS size_bytes FROM documents WHERE folder_id = $1 AND owner_id = (SELECT f.owner_id FROM folders f WHERE f.id = $2) ORDER BY updated_at DESC")
        )
        .bind(&folder_id)
        .bind(&folder_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        sqlx::query_as::<_, Document>(
            sql("SELECT id, owner_id, folder_id, title, thumbnail_svg, public_role, created_at, updated_at, CAST(COALESCE(LENGTH(content), 0) AS BIGINT) AS size_bytes FROM documents WHERE owner_id = $1 AND folder_id IS NULL ORDER BY updated_at DESC")
        )
        .bind(&user_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    Ok(Json(docs))
}

#[axum::debug_handler]
pub async fn create_document(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<CreateDocumentRequest>,
) -> Result<Json<Document>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;
    crate::sharing::access::require_owned_folder(&state, &payload.folder_id, &user_id).await?;

    let doc_id = Uuid::new_v4().to_string();

    let initial_text = payload.content.as_deref().unwrap_or("== New Document");
    let content = encode_text_blob(initial_text);
    crate::accounts::storage::ensure_room(&state, &user_id, content.len() as i64).await?;

    sqlx::query(sql("INSERT INTO documents (id, owner_id, folder_id, title, content) VALUES ($1, $2, $3, $4, $5)"))
    .bind(&doc_id)
    .bind(&user_id)
    .bind(&payload.folder_id)
    .bind(&payload.title)
    .bind(&content)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let doc = sqlx::query_as::<_, Document>(sql(DOCUMENT_BY_ID))
    .bind(&doc_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(doc))
}

pub async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<Document>, (StatusCode, String)> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());

    let (mut doc, role) = crate::sharing::access::document_role(&state, &id, &user_id_opt)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    doc.effective_role = Some(role.as_str().to_string());
    Ok(Json(doc))
}

pub async fn update_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    Json(payload): Json<crate::models::UpdateDocumentRequest>,
) -> Result<Json<Document>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let mut doc = sqlx::query_as::<_, Document>(
        sql(OWNED_DOCUMENT)
    )
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;

    if let Some(new_title) = payload.title {
        doc.title = new_title;
    }
    if let Some(new_folder_id) = payload.folder_id {
        crate::sharing::access::require_owned_folder(&state, &Some(new_folder_id.clone()), &user_id).await?;
        if new_folder_id.is_empty() {
            doc.folder_id = None;
        } else {
            doc.folder_id = Some(new_folder_id);
        }
    }
    if let Some(new_public_role) = payload.public_role {
        doc.public_role = crate::sharing::links::parse_public_role(&new_public_role, true)?;
    }

    sqlx::query(sql("UPDATE documents SET title = $1, folder_id = $2, public_role = $3, updated_at = $4 WHERE id = $5 AND owner_id = $6"))
    .bind(&doc.title)
    .bind(&doc.folder_id)
    .bind(&doc.public_role)
    .bind(crate::db::timestamp_now())
    .bind(&id)
    .bind(&user_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let doc = sqlx::query_as::<_, Document>(sql(OWNED_DOCUMENT))
    .bind(&id)
    .bind(&user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::sharing::access::revalidate_resource(&state, crate::sharing::access::Resource::Document(id.clone())).await;

    Ok(Json(doc))
}

pub async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

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

    Ok(StatusCode::NO_CONTENT)
}

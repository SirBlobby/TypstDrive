use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use crate::accounts::signed_in::SignedInUser;
use serde::Serialize;

use crate::db::sql;
use crate::sharing::access::{self, AccessRole};
use crate::AppState;

type ApiError = (StatusCode, String);

#[derive(Serialize)]
pub struct LibraryImage {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub size_bytes: i64,
}

#[derive(Serialize)]
pub struct LibraryDocument {
    pub id: String,
    pub title: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct DocumentText {
    pub title: String,
    pub text: String,
}

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

pub async fn list_images(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<Json<Vec<LibraryImage>>, ApiError> {
    let user = Some(user_id);
    let (document, role) = access::document_role(&state, &document_id, &user).await.ok_or_else(access::forbidden)?;

    type ImageRow = (String, String, String, i64, String, Option<String>, i64);
    let rows: Vec<ImageRow> = sqlx::query_as(sql(
        "SELECT id, name, mime_type, size_bytes, owner_id, folder_id, \
         CAST(CASE WHEN document_id = $1 OR id IN (SELECT file_id FROM document_shared_files WHERE document_id = $2) THEN 1 ELSE 0 END AS BIGINT) \
         FROM files WHERE (document_id = $3 OR owner_id = $4) AND deleted_at IS NULL AND mime_type LIKE 'image/%' ORDER BY name ASC",
    ))
    .bind(&document.id)
    .bind(&document.id)
    .bind(&document.id)
    .bind(&document.owner_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let mut images = Vec::new();
    for (id, name, mime_type, size_bytes, owner_id, folder_id, linked) in rows {
        let usable = role == AccessRole::Owner
            || linked != 0
            || (folder_id.is_some() && access::inherited_role(&state, &folder_id, &owner_id, &user).await.is_some());
        if usable {
            images.push(LibraryImage { id, name, mime_type, size_bytes });
        }
    }
    Ok(Json(images))
}

pub async fn list_documents(
    State(state): State<AppState>,
    SignedInUser(user_id): SignedInUser,
) -> Result<Json<Vec<LibraryDocument>>, ApiError> {
    let documents = sqlx::query_as::<_, (String, String, String)>(sql(
        "SELECT d.id, d.title, d.updated_at FROM documents d WHERE (d.owner_id = $1 \
         OR d.id IN (SELECT c.document_id FROM collaborators c WHERE c.user_id = $2)) \
         AND d.deleted_at IS NULL ORDER BY d.updated_at DESC",
    ))
    .bind(&user_id)
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?
    .into_iter()
    .map(|(id, title, updated_at)| LibraryDocument { id, title, updated_at })
    .collect();
    Ok(Json(documents))
}

pub async fn document_text(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<Json<DocumentText>, ApiError> {
    let user = Some(user_id);
    let (document, _) = access::document_role(&state, &document_id, &user).await.ok_or_else(access::forbidden)?;
    let room_id = crate::realtime::live_documents::document_room(&document_id);
    let text = match crate::realtime::live_documents::live_text(&state, &room_id).await {
        Some(text) => text,
        None => crate::drive::documents::saved_text(&state, &document_id).await,
    };
    Ok(Json(DocumentText { title: document.title, text }))
}

use axum::{
    extract::{Path, State, Multipart},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use uuid::Uuid;
use yrs::{Doc, ReadTxn, Transact, Text};

use crate::{
    models::{Document, CreateDocumentRequest},
    AppState,
};

#[derive(serde::Deserialize)]
pub struct ListDocsQuery {
    pub folder_id: Option<String>,
}

pub async fn list_documents(
    axum::extract::Query(query): axum::extract::Query<ListDocsQuery>,
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<Document>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let docs = if let Some(folder_id) = query.folder_id {
        sqlx::query_as::<_, Document>(
            "SELECT id, owner_id, folder_id, title, content, thumbnail_svg, public_role, created_at, updated_at FROM documents WHERE owner_id = $1 AND folder_id = $2 ORDER BY updated_at DESC"
        )
        .bind(&user_id)
        .bind(&folder_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        sqlx::query_as::<_, Document>(
            "SELECT id, owner_id, folder_id, title, content, thumbnail_svg, public_role, created_at, updated_at FROM documents WHERE owner_id = $1 AND folder_id IS NULL ORDER BY updated_at DESC"
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

    let doc_id = Uuid::new_v4().to_string();
    
    let content = {
        let ydoc = Doc::new();
        let text = ydoc.get_or_insert_text("typst");
        let initial_text = payload.content.clone().unwrap_or_else(|| "== New Document".to_string());
        println!("Creating document with content length: {}", initial_text.len());
        text.insert(&mut ydoc.transact_mut(), 0, &initial_text);
        let encoded = ydoc.transact().encode_state_as_update_v1(&yrs::StateVector::default());
        println!("Encoded Yjs state length: {}", encoded.len());
        encoded
    };

    let doc = sqlx::query_as::<_, Document>(
        "INSERT INTO documents (id, owner_id, folder_id, title, content) VALUES ($1, $2, $3, $4, $5) RETURNING id, owner_id, folder_id, title, content, thumbnail_svg, public_role, created_at, updated_at"
    )
    .bind(&doc_id)
    .bind(&user_id)
    .bind(&payload.folder_id)
    .bind(&payload.title)
    .bind(&content)
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

    let mut doc = sqlx::query_as::<_, Document>(
        "SELECT id, owner_id, folder_id, title, content, thumbnail_svg, public_role, created_at, updated_at FROM documents WHERE id = $1"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;

    let mut effective_role = "none".to_string();

    if let Some(uid) = &user_id_opt {
        if &doc.owner_id == uid {
            effective_role = "owner".to_string();
        } else {
            if let Ok(Some((role,))) = sqlx::query_as::<_, (String,)>("SELECT role FROM collaborators WHERE document_id = $1 AND user_id = $2")
                .bind(&id)
                .bind(uid)
                .fetch_optional(&state.db)
                .await 
            {
                effective_role = role;
            }
        }
    }

    if effective_role == "none" {
        if let Some(pr) = &doc.public_role {
            if pr == "viewer" || pr == "editor" {
                effective_role = pr.clone();
            }
        }
    }

    if effective_role == "none" {
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    }

    doc.effective_role = Some(effective_role);
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
        "SELECT id, owner_id, folder_id, title, content, thumbnail_svg, public_role, created_at, updated_at FROM documents WHERE id = $1 AND owner_id = $2"
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
        if new_folder_id.is_empty() {
            doc.folder_id = None;
        } else {
            doc.folder_id = Some(new_folder_id);
        }
    }
    if let Some(new_public_role) = payload.public_role {
        if new_public_role == "none" || new_public_role.is_empty() {
            doc.public_role = None;
        } else {
            doc.public_role = Some(new_public_role);
        }
    }

    
    let doc = sqlx::query_as::<_, Document>(
        "UPDATE documents SET title = $1, folder_id = $2, public_role = $3, updated_at = CURRENT_TIMESTAMP WHERE id = $4 AND owner_id = $5 RETURNING id, owner_id, folder_id, title, content, thumbnail_svg, public_role, created_at, updated_at"
    )
    .bind(&doc.title)
    .bind(&doc.folder_id)
    .bind(&doc.public_role)
    .bind(&id)
    .bind(&user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(doc))
}

pub async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let result = sqlx::query("DELETE FROM documents WHERE id = $1 AND owner_id = $2")
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Document not found or unauthorized".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn upload_file(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    jar: SignedCookieJar,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    
    let doc_exists = sqlx::query_as::<_, (String, Option<String>)>("SELECT id, folder_id FROM documents WHERE id = $1 AND owner_id = $2")
        .bind(&doc_id)
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if doc_exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "Document not found or unauthorized".to_string()));
    }

    let (_, folder_id) = doc_exists.unwrap();

    let mut uploaded_filename = String::new();
    let mut font_family = None;

    if let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        let file_name = field.file_name().unwrap_or("unnamed").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data = field.bytes().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?.to_vec();

        let file_id = Uuid::new_v4().to_string();

        if file_name.to_lowercase().ends_with(".ttf") || file_name.to_lowercase().ends_with(".otf") {
            if let Some(font) = typst::text::Font::iter(typst::foundations::Bytes::new(data.clone())).next() {
                font_family = Some(font.info().family.clone());
            }
        }

        sqlx::query("INSERT INTO files (id, owner_id, document_id, folder_id, name, mime_type, data) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(&file_id)
            .bind(&user_id)
            .bind(&doc_id)
            .bind(&folder_id)
            .bind(&file_name)
            .bind(&content_type)
            .bind(&data)
            .execute(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        uploaded_filename = file_name;
    }

    Ok(Json(serde_json::json!({
        "filename": uploaded_filename,
        "font_family": font_family
    })))
}

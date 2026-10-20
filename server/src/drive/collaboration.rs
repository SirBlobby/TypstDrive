use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use uuid::Uuid;

use crate::{
    sharing::access,
    models::{Comment, CreateCommentRequest, UpdateCommentRequest},
    AppState,
};
use crate::db::sql;

pub async fn get_comments(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<Comment>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    access::document_role(&state, &doc_id, &Some(user_id))
        .await
        .ok_or_else(access::forbidden)?;

    let comments = sqlx::query_as::<_, Comment>(
        sql("SELECT c.id, c.document_id, c.user_id, c.content, c.resolved, c.created_at, u.username as author_name \
         FROM comments c \
         LEFT JOIN users u ON c.user_id = u.id \
         WHERE c.document_id = $1 \
         ORDER BY c.created_at ASC")
    )
    .bind(&doc_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(comments))
}

pub async fn add_comment(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    jar: SignedCookieJar,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<Json<Comment>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let (_, role) = access::document_role(&state, &doc_id, &Some(user_id.clone()))
        .await
        .ok_or_else(access::forbidden)?;
    if !role.can_edit() {
        return Err(access::forbidden());
    }

    let comment_id = Uuid::new_v4().to_string();

    sqlx::query(sql("INSERT INTO comments (id, document_id, user_id, content) VALUES ($1, $2, $3, $4)"))
        .bind(&comment_id)
        .bind(&doc_id)
        .bind(&user_id)
        .bind(&payload.content)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let comment = sqlx::query_as::<_, Comment>(
        sql("SELECT c.id, c.document_id, c.user_id, c.content, c.resolved, c.created_at, u.username as author_name \
         FROM comments c \
         LEFT JOIN users u ON c.user_id = u.id \
         WHERE c.id = $1")
    )
    .bind(&comment_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(comment))
}

pub async fn create_version(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    jar: SignedCookieJar,
    Json(payload): Json<crate::models::CreateVersionRequest>,
) -> Result<Json<crate::models::DocumentVersion>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let (_, role) = access::document_role(&state, &doc_id, &Some(user_id.clone()))
        .await
        .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;

    if !role.can_edit() {
        return Err((StatusCode::FORBIDDEN, "Not authorized to create versions".to_string()));
    }

    let version_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(sql("INSERT INTO document_versions (id, document_id, user_id, content) VALUES ($1, $2, $3, $4)"))
        .bind(&version_id)
        .bind(&doc_id)
        .bind(&user_id)
        .bind(&payload.content)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let version = sqlx::query_as::<_, crate::models::DocumentVersion>(
        sql("SELECT v.id, v.document_id, v.user_id, v.content, v.created_at, u.username as author_name \
         FROM document_versions v \
         LEFT JOIN users u ON v.user_id = u.id \
         WHERE v.id = $1")
    )
    .bind(&version_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(version))
}

pub async fn get_versions(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<crate::models::DocumentVersion>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    access::document_role(&state, &doc_id, &Some(user_id))
        .await
        .ok_or_else(access::forbidden)?;

    let versions = sqlx::query_as::<_, crate::models::DocumentVersion>(
        sql("SELECT v.id, v.document_id, v.user_id, v.content, v.created_at, u.username as author_name \
         FROM document_versions v \
         LEFT JOIN users u ON v.user_id = u.id \
         WHERE v.document_id = $1 \
         ORDER BY v.created_at DESC")
    )
    .bind(&doc_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(versions))
}

pub async fn update_comment(
    State(state): State<AppState>,
    Path(comment_id): Path<String>,
    jar: SignedCookieJar,
    Json(payload): Json<UpdateCommentRequest>,
) -> Result<Json<Comment>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let mut comment = sqlx::query_as::<_, Comment>(
        sql("SELECT c.id, c.document_id, c.user_id, c.content, c.resolved, c.created_at, u.username as author_name \
         FROM comments c \
         LEFT JOIN users u ON c.user_id = u.id \
         WHERE c.id = $1 AND c.user_id = $2")
    )
    .bind(&comment_id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Comment not found or unauthorized".to_string()))?;

    if let Some(c) = payload.content {
        comment.content = c;
    }
    if let Some(resolved) = payload.resolved {
        comment.resolved = if resolved { 1 } else { 0 };
    }

    sqlx::query(sql("UPDATE comments SET content = $1, resolved = $2 WHERE id = $3"))
        .bind(&comment.content)
        .bind(comment.resolved)
        .bind(&comment.id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let updated_comment = sqlx::query_as::<_, Comment>(
        sql("SELECT c.id, c.document_id, c.user_id, c.content, c.resolved, c.created_at, u.username as author_name \
         FROM comments c \
         LEFT JOIN users u ON c.user_id = u.id \
         WHERE c.id = $1")
    )
    .bind(&comment.id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(updated_comment))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    Path(comment_id): Path<String>,
    jar: SignedCookieJar,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let result = sqlx::query(sql("DELETE FROM comments WHERE id = $1 AND user_id = $2"))
        .bind(&comment_id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Comment not found or unauthorized".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

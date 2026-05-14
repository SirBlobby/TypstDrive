use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    models::{Collaborator, Comment, CreateCommentRequest, Invitation, InviteRequest, UpdateCommentRequest},
    AppState,
};

pub async fn invite_collaborator(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    jar: SignedCookieJar,
    Json(payload): Json<InviteRequest>,
) -> Result<Json<Invitation>, (StatusCode, String)> {
    let inviter_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let doc_exists = sqlx::query_as::<_, (String,)>("SELECT id FROM documents WHERE id = ? AND owner_id = ?")
        .bind(&doc_id)
        .bind(&inviter_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if doc_exists.is_none() {
        return Err((StatusCode::FORBIDDEN, "Only the owner can invite collaborators".to_string()));
    }

    let invited_user = sqlx::query_as::<_, crate::models::User>("SELECT id, username, email, password_hash FROM users WHERE email = ?")
        .bind(&payload.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(user) = invited_user {
        let collab_id = Uuid::new_v4().to_string();
        let _collab = sqlx::query_as::<_, Collaborator>(
            "INSERT INTO collaborators (id, document_id, user_id, role) VALUES (?, ?, ?, ?) ON CONFLICT (document_id, user_id) DO UPDATE SET role = excluded.role RETURNING id, document_id, user_id, role, created_at"
        )
        .bind(&collab_id)
        .bind(&doc_id)
        .bind(&user.id)
        .bind(&payload.role)
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let inv = Invitation {
            id: Uuid::new_v4().to_string(),
            document_id: doc_id.to_string(),
            role: payload.role.clone(),
            token: "direct-added".to_string(),
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            expires_at: None,
        };
        Ok(Json(inv))
    } else {
        Err((StatusCode::NOT_FOUND, "User with that email not found".to_string()))
    }
}

#[derive(Deserialize)]
pub struct AcceptInviteQuery {
    pub token: String,
}

pub async fn accept_invite(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Query(query): Query<AcceptInviteQuery>,
) -> Result<Json<Collaborator>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let invitation = sqlx::query_as::<_, Invitation>(
        "SELECT id, document_id, role, token, created_at, expires_at FROM invitations WHERE token = ?"
    )
    .bind(&query.token)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Invalid or expired invitation".to_string()))?;

    let collab_id = Uuid::new_v4().to_string();

    let collab = sqlx::query_as::<_, Collaborator>(
        "INSERT INTO collaborators (id, document_id, user_id, role) VALUES (?, ?, ?, ?) ON CONFLICT (document_id, user_id) DO UPDATE SET role = excluded.role RETURNING id, document_id, user_id, role, created_at"
    )
    .bind(&collab_id)
    .bind(&invitation.document_id)
    .bind(&user_id)
    .bind(&invitation.role)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(collab))
}

pub async fn get_comments(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<Comment>>, (StatusCode, String)> {
    let _user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let comments = sqlx::query_as::<_, Comment>(
        "SELECT c.id, c.document_id, c.user_id, c.content, c.resolved, c.created_at, u.username as author_name \
         FROM comments c \
         LEFT JOIN users u ON c.user_id = u.id \
         WHERE c.document_id = ? \
         ORDER BY c.created_at ASC"
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

    let comment_id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO comments (id, document_id, user_id, content) VALUES (?, ?, ?, ?)")
        .bind(&comment_id)
        .bind(&doc_id)
        .bind(&user_id)
        .bind(&payload.content)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let comment = sqlx::query_as::<_, Comment>(
        "SELECT c.id, c.document_id, c.user_id, c.content, c.resolved, c.created_at, u.username as author_name \
         FROM comments c \
         LEFT JOIN users u ON c.user_id = u.id \
         WHERE c.id = ?"
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

    let doc = sqlx::query_as::<_, crate::models::Document>(
        "SELECT id, owner_id, folder_id, title, content, thumbnail_svg, public_role, created_at, updated_at FROM documents WHERE id = ?"
    )
    .bind(&doc_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;

    let is_owner = doc.owner_id == user_id;
    let role = sqlx::query_scalar::<_, String>("SELECT role FROM collaborators WHERE document_id = ? AND user_id = ?")
        .bind(&doc_id)
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !is_owner && role != Some("editor".to_string()) {
        return Err((StatusCode::FORBIDDEN, "Not authorized to create versions".to_string()));
    }

    let version_id = uuid::Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO document_versions (id, document_id, user_id, content) VALUES (?, ?, ?, ?)")
        .bind(&version_id)
        .bind(&doc_id)
        .bind(&user_id)
        .bind(&payload.content)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let version = sqlx::query_as::<_, crate::models::DocumentVersion>(
        "SELECT v.id, v.document_id, v.user_id, v.content, v.created_at, u.username as author_name \
         FROM document_versions v \
         LEFT JOIN users u ON v.user_id = u.id \
         WHERE v.id = ?"
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
    let _user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let versions = sqlx::query_as::<_, crate::models::DocumentVersion>(
        "SELECT v.id, v.document_id, v.user_id, v.content, v.created_at, u.username as author_name \
         FROM document_versions v \
         LEFT JOIN users u ON v.user_id = u.id \
         WHERE v.document_id = ? \
         ORDER BY v.created_at DESC"
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
        "SELECT c.id, c.document_id, c.user_id, c.content, c.resolved, c.created_at, u.username as author_name \
         FROM comments c \
         LEFT JOIN users u ON c.user_id = u.id \
         WHERE c.id = ? AND c.user_id = ?"
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
    if let Some(r) = payload.resolved {
        comment.resolved = r;
    }

    sqlx::query("UPDATE comments SET content = ?, resolved = ? WHERE id = ?")
        .bind(&comment.content)
        .bind(comment.resolved)
        .bind(&comment.id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let updated_comment = sqlx::query_as::<_, Comment>(
        "SELECT c.id, c.document_id, c.user_id, c.content, c.resolved, c.created_at, u.username as author_name \
         FROM comments c \
         LEFT JOIN users u ON c.user_id = u.id \
         WHERE c.id = ?"
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

    let result = sqlx::query("DELETE FROM comments WHERE id = ? AND user_id = ?")
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

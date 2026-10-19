use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use uuid::Uuid;

use crate::{
    access::{self, Resource},
    models::{Collaborator, CollaboratorView, Comment, CreateCommentRequest, Invitation, InviteRequest, UpdateCommentRequest},
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

    let doc_exists = sqlx::query_as::<_, (String,)>("SELECT id FROM documents WHERE id = $1 AND owner_id = $2")
        .bind(&doc_id)
        .bind(&inviter_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if doc_exists.is_none() {
        return Err((StatusCode::FORBIDDEN, "Only the owner can invite collaborators".to_string()));
    }

    if payload.role != "viewer" && payload.role != "editor" {
        return Err((StatusCode::BAD_REQUEST, "Role must be viewer or editor".to_string()));
    }

    let invited_user = sqlx::query_as::<_, crate::models::User>("SELECT id, username, email, password_hash, is_admin FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(user) = invited_user {
        let collab_id = Uuid::new_v4().to_string();
        let _collab = sqlx::query_as::<_, Collaborator>(
            "INSERT INTO collaborators (id, document_id, user_id, role) VALUES ($1, $2, $3, $4) ON CONFLICT (document_id, user_id) DO UPDATE SET role = excluded.role RETURNING id, document_id, user_id, role, created_at"
        )
        .bind(&collab_id)
        .bind(&doc_id)
        .bind(&user.id)
        .bind(&payload.role)
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        access::revalidate_resource(&state, Resource::Document(doc_id.clone())).await;

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

pub async fn list_collaborators(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<CollaboratorView>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    // Only owner or collaborators on the document can see the list
    let has_access = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM documents WHERE id = $1 AND owner_id = $2 \
         UNION ALL SELECT COUNT(*) FROM collaborators WHERE document_id = $3 AND user_id = $4"
    )
    .bind(&doc_id).bind(&user_id).bind(&doc_id).bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .into_iter().sum::<i64>() > 0;

    if !has_access {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let collaborators = sqlx::query_as::<_, CollaboratorView>(
        "SELECT c.id, c.user_id, u.username, u.email, c.role, c.created_at \
         FROM collaborators c \
         INNER JOIN users u ON u.id = c.user_id \
         WHERE c.document_id = $1 \
         ORDER BY c.created_at ASC"
    )
    .bind(&doc_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(collaborators))
}

pub async fn remove_collaborator(
    State(state): State<AppState>,
    Path((doc_id, collab_id)): Path<(String, String)>,
    jar: SignedCookieJar,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    // Only the document owner can remove collaborators
    let is_owner = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM documents WHERE id = $1 AND owner_id = $2"
    )
    .bind(&doc_id)
    .bind(&user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))? > 0;

    if !is_owner {
        return Err((StatusCode::FORBIDDEN, "Only the document owner can remove collaborators".to_string()));
    }

    let result = sqlx::query(
        "DELETE FROM collaborators WHERE id = $1 AND document_id = $2"
    )
    .bind(&collab_id)
    .bind(&doc_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Collaborator not found".to_string()));
    }

    access::revalidate_resource(&state, Resource::Document(doc_id.clone())).await;

    Ok(StatusCode::NO_CONTENT)
}

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
        "SELECT c.id, c.document_id, c.user_id, c.content, c.resolved, c.created_at, u.username as author_name \
         FROM comments c \
         LEFT JOIN users u ON c.user_id = u.id \
         WHERE c.document_id = $1 \
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

    let (_, role) = access::document_role(&state, &doc_id, &Some(user_id.clone()))
        .await
        .ok_or_else(access::forbidden)?;
    if !role.can_edit() {
        return Err(access::forbidden());
    }

    let comment_id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO comments (id, document_id, user_id, content) VALUES ($1, $2, $3, $4)")
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
         WHERE c.id = $1"
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

    sqlx::query("INSERT INTO document_versions (id, document_id, user_id, content) VALUES ($1, $2, $3, $4)")
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
         WHERE v.id = $1"
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
        "SELECT v.id, v.document_id, v.user_id, v.content, v.created_at, u.username as author_name \
         FROM document_versions v \
         LEFT JOIN users u ON v.user_id = u.id \
         WHERE v.document_id = $1 \
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
         WHERE c.id = $1 AND c.user_id = $2"
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

    sqlx::query("UPDATE comments SET content = $1, resolved = $2 WHERE id = $3")
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
         WHERE c.id = $1"
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

    let result = sqlx::query("DELETE FROM comments WHERE id = $1 AND user_id = $2")
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

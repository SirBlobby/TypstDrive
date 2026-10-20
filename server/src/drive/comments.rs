use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use crate::accounts::signed_in::SignedInUser;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::{sql, sql_owned, timestamp_now};
use crate::drive::people::{load_people, PersonSummary};
use crate::sharing::access::{self, AccessRole};
use crate::AppState;

type ApiError = (StatusCode, String);

const MAX_COMMENT_LENGTH: usize = 10_000;
const MAX_QUOTE_LENGTH: usize = 500;

#[derive(Serialize)]
pub struct CommentView {
    pub id: String,
    pub document_id: String,
    pub parent_id: Option<String>,
    pub author: Option<PersonSummary>,
    pub user_id: String,
    pub content: String,
    pub resolved: bool,
    pub anchor_start: Option<String>,
    pub anchor_end: Option<String>,
    pub quoted_text: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
    pub parent_id: Option<String>,
    pub anchor_start: Option<String>,
    pub anchor_end: Option<String>,
    pub quoted_text: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateCommentRequest {
    pub content: Option<String>,
    pub resolved: Option<bool>,
}

type CommentRow = (String, String, Option<String>, String, String, i64, Option<String>, Option<String>, Option<String>, String, Option<String>);

const COMMENT_COLUMNS: &str = "SELECT id, document_id, parent_id, user_id, content, resolved, anchor_start, anchor_end, quoted_text, created_at, updated_at FROM comments";

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn not_found() -> ApiError {
    (StatusCode::NOT_FOUND, "Comment not found".to_string())
}

fn clean_content(content: &str) -> Result<String, ApiError> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Comment cannot be empty".to_string()));
    }
    if trimmed.len() > MAX_COMMENT_LENGTH {
        return Err((StatusCode::BAD_REQUEST, "Comment is too long".to_string()));
    }
    Ok(trimmed.to_string())
}

async fn document_access(state: &AppState, document_id: &str, user_id: &str) -> Result<AccessRole, ApiError> {
    access::document_role(state, document_id, &Some(user_id.to_string()))
        .await
        .map(|(_, role)| role)
        .ok_or_else(access::forbidden)
}

async fn to_views(state: &AppState, rows: Vec<CommentRow>) -> Vec<CommentView> {
    let author_ids: Vec<String> = rows.iter().map(|row| row.3.clone()).collect();
    let directory = load_people(state, &author_ids).await;
    rows.into_iter()
        .map(|(id, document_id, parent_id, user_id, content, resolved, anchor_start, anchor_end, quoted_text, created_at, updated_at)| CommentView {
            author: directory.get(&user_id).cloned(),
            id,
            document_id,
            parent_id,
            user_id,
            content,
            resolved: resolved != 0,
            anchor_start,
            anchor_end,
            quoted_text,
            created_at,
            updated_at,
        })
        .collect()
}

async fn load_comment(state: &AppState, comment_id: &str) -> Result<CommentView, ApiError> {
    let row = sqlx::query_as::<_, CommentRow>(sql_owned(format!("{COMMENT_COLUMNS} WHERE id = $1")))
        .bind(comment_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .ok_or_else(not_found)?;
    to_views(state, vec![row]).await.pop().ok_or_else(not_found)
}

async fn thread_root(state: &AppState, document_id: &str, parent_id: &str) -> Result<String, ApiError> {
    let (id, grandparent_id) = sqlx::query_as::<_, (String, Option<String>)>(sql(
        "SELECT id, parent_id FROM comments WHERE id = $1 AND document_id = $2",
    ))
    .bind(parent_id)
    .bind(document_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or_else(not_found)?;
    Ok(grandparent_id.unwrap_or(id))
}

pub async fn get_comments(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<Json<Vec<CommentView>>, ApiError> {
    document_access(&state, &document_id, &user_id).await?;
    let rows = sqlx::query_as::<_, CommentRow>(sql_owned(format!("{COMMENT_COLUMNS} WHERE document_id = $1 ORDER BY created_at ASC")))
        .bind(&document_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(Json(to_views(&state, rows).await))
}

pub async fn add_comment(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<Json<CommentView>, ApiError> {
    if !document_access(&state, &document_id, &user_id).await?.can_comment() {
        return Err(access::forbidden());
    }

    let content = clean_content(&payload.content)?;
    let parent_id = match payload.parent_id.as_deref() {
        Some(parent_id) => Some(thread_root(&state, &document_id, parent_id).await?),
        None => None,
    };
    let is_reply = parent_id.is_some();
    let quoted_text = payload.quoted_text.filter(|_| !is_reply).map(|quote| quote.chars().take(MAX_QUOTE_LENGTH).collect::<String>());

    let comment_id = Uuid::new_v4().to_string();
    sqlx::query(sql(
        "INSERT INTO comments (id, document_id, user_id, content, parent_id, anchor_start, anchor_end, quoted_text, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    ))
    .bind(&comment_id)
    .bind(&document_id)
    .bind(&user_id)
    .bind(&content)
    .bind(&parent_id)
    .bind(payload.anchor_start.filter(|_| !is_reply))
    .bind(payload.anchor_end.filter(|_| !is_reply))
    .bind(quoted_text)
    .bind(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string())
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    if let Some(root_id) = &parent_id {
        sqlx::query(sql("UPDATE comments SET resolved = 0 WHERE id = $1"))
            .bind(root_id)
            .execute(&state.db)
            .await
            .map_err(internal_error)?;
    }

    Ok(Json(load_comment(&state, &comment_id).await?))
}

pub async fn update_comment(
    State(state): State<AppState>,
    Path(comment_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<UpdateCommentRequest>,
) -> Result<Json<CommentView>, ApiError> {
    let comment = load_comment(&state, &comment_id).await?;
    let role = document_access(&state, &comment.document_id, &user_id).await?;
    let is_author = comment.user_id == user_id;

    if let Some(content) = payload.content {
        if !is_author {
            return Err((StatusCode::FORBIDDEN, "Only the author can edit a comment".to_string()));
        }
        sqlx::query(sql("UPDATE comments SET content = $1, updated_at = $2 WHERE id = $3"))
            .bind(clean_content(&content)?)
            .bind(timestamp_now())
            .bind(&comment_id)
            .execute(&state.db)
            .await
            .map_err(internal_error)?;
    }

    if let Some(resolved) = payload.resolved {
        if !role.can_comment() {
            return Err(access::forbidden());
        }
        if comment.parent_id.is_some() {
            return Err((StatusCode::BAD_REQUEST, "Resolve the whole thread instead of a reply".to_string()));
        }
        sqlx::query(sql("UPDATE comments SET resolved = $1 WHERE id = $2"))
            .bind(if resolved { 1i64 } else { 0i64 })
            .bind(&comment_id)
            .execute(&state.db)
            .await
            .map_err(internal_error)?;
    }

    Ok(Json(load_comment(&state, &comment_id).await?))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    Path(comment_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<StatusCode, ApiError> {
    let comment = load_comment(&state, &comment_id).await?;
    let role = document_access(&state, &comment.document_id, &user_id).await?;
    if comment.user_id != user_id && role != AccessRole::Owner {
        return Err((StatusCode::FORBIDDEN, "Only the author or the document owner can delete a comment".to_string()));
    }

    sqlx::query(sql("DELETE FROM comments WHERE id = $1 OR parent_id = $2"))
        .bind(&comment_id)
        .bind(&comment_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

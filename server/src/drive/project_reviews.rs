use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::accounts::signed_in::SignedInUser;
use crate::drive::comments::{self, CommentView, CreateCommentRequest, UpdateCommentRequest};
use crate::drive::review_target::{project_file_target, project_role, ReviewKind};
use crate::drive::suggestions::{self, CreateSuggestionRequest, SuggestionView, UpdateSuggestionRequest};
use crate::AppState;

type ApiError = (StatusCode, String);

fn internal_error(error: sqlx::Error) -> ApiError {
    crate::server_errors::hidden(error)
}

pub async fn list_comments(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<Json<Vec<CommentView>>, ApiError> {
    project_role(&state, &project_id, &user_id).await?;
    Ok(Json(comments::project_comments(&state, &project_id).await.map_err(internal_error)?))
}

pub async fn add_comment(
    State(state): State<AppState>,
    Path((project_id, file_id)): Path<(String, String)>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<Json<CommentView>, ApiError> {
    let target = project_file_target(&state, &project_id, &file_id).await?;
    Ok(Json(comments::create_comment(&state, target, &user_id, payload).await?))
}

pub async fn update_comment(
    State(state): State<AppState>,
    Path(comment_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<UpdateCommentRequest>,
) -> Result<Json<CommentView>, ApiError> {
    Ok(Json(comments::change_comment(&state, ReviewKind::ProjectFile, &comment_id, &user_id, payload).await?))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    Path(comment_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<StatusCode, ApiError> {
    comments::remove_comment(&state, ReviewKind::ProjectFile, &comment_id, &user_id).await
}

pub async fn list_suggestions(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<Json<Vec<SuggestionView>>, ApiError> {
    project_role(&state, &project_id, &user_id).await?;
    Ok(Json(suggestions::project_suggestions(&state, &project_id).await.map_err(internal_error)?))
}

pub async fn create_suggestion(
    State(state): State<AppState>,
    Path((project_id, file_id)): Path<(String, String)>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<CreateSuggestionRequest>,
) -> Result<Json<SuggestionView>, ApiError> {
    let target = project_file_target(&state, &project_id, &file_id).await?;
    Ok(Json(suggestions::create_suggestion_for(&state, target, &user_id, payload).await?))
}

pub async fn update_suggestion(
    State(state): State<AppState>,
    Path(suggestion_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<UpdateSuggestionRequest>,
) -> Result<Json<SuggestionView>, ApiError> {
    Ok(Json(suggestions::change_suggestion(&state, ReviewKind::ProjectFile, &suggestion_id, &user_id, payload).await?))
}

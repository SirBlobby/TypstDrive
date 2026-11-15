use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use crate::accounts::signed_in::SignedInUser;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::{sql_owned, timestamp_now};
use crate::drive::people::{load_people, PersonSummary};
use crate::drive::review_target::{review_access, ReviewKind, ReviewTarget};
use crate::sharing::access;
use crate::AppState;

type ApiError = (StatusCode, String);

const MAX_TEXT_LENGTH: usize = 20_000;
const PENDING: &str = "pending";
const ACCEPTED: &str = "accepted";
const REJECTED: &str = "rejected";
const WITHDRAWN: &str = "withdrawn";

#[derive(Serialize)]
pub struct SuggestionView {
    pub id: String,
    #[serde(flatten)]
    pub target: ReviewTarget,
    pub user_id: String,
    pub author: Option<PersonSummary>,
    pub anchor_start: String,
    pub anchor_end: String,
    pub original_text: String,
    pub replacement_text: String,
    pub note: Option<String>,
    pub status: String,
    pub resolved_by: Option<PersonSummary>,
    pub resolved_at: Option<String>,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct CreateSuggestionRequest {
    pub anchor_start: String,
    pub anchor_end: String,
    pub original_text: String,
    pub replacement_text: String,
    pub note: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateSuggestionRequest {
    pub status: Option<String>,
    pub content: Option<SuggestionContent>,
    pub expected_replacement_text: Option<String>,
}

#[derive(Deserialize)]
pub struct SuggestionContent {
    pub anchor_start: String,
    pub anchor_end: String,
    pub original_text: String,
    pub replacement_text: String,
}

type SuggestionRow = (String, String, String, String, String, String, String, Option<String>, String, Option<String>, Option<String>, String);

fn select_suggestions(kind: ReviewKind) -> String {
    format!(
        "SELECT id, {}, user_id, anchor_start, anchor_end, original_text, replacement_text, note, status, resolved_by, resolved_at, created_at FROM {}",
        kind.target_column(),
        kind.suggestions_table()
    )
}

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn not_found() -> ApiError {
    (StatusCode::NOT_FOUND, "Suggestion not found".to_string())
}

async fn to_views(state: &AppState, kind: ReviewKind, rows: Vec<SuggestionRow>) -> Vec<SuggestionView> {
    let mut ids: Vec<String> = rows.iter().map(|row| row.2.clone()).collect();
    ids.extend(rows.iter().filter_map(|row| row.9.clone()));
    let directory = load_people(state, &ids).await;
    rows.into_iter()
        .map(|(id, target_id, user_id, anchor_start, anchor_end, original_text, replacement_text, note, status, resolved_by, resolved_at, created_at)| SuggestionView {
            author: directory.get(&user_id).cloned(),
            resolved_by: resolved_by.and_then(|resolver_id| directory.get(&resolver_id).cloned()),
            target: ReviewTarget::new(kind, target_id),
            id,
            user_id,
            anchor_start,
            anchor_end,
            original_text,
            replacement_text,
            note,
            status,
            resolved_at,
            created_at,
        })
        .collect()
}

async fn load_suggestion(state: &AppState, kind: ReviewKind, suggestion_id: &str) -> Result<SuggestionView, ApiError> {
    let row = sqlx::query_as::<_, SuggestionRow>(sql_owned(format!("{} WHERE id = $1", select_suggestions(kind))))
        .bind(suggestion_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .ok_or_else(not_found)?;
    to_views(state, kind, vec![row]).await.pop().ok_or_else(not_found)
}

pub async fn document_suggestions(state: &AppState, document_id: &str) -> Result<Vec<SuggestionView>, sqlx::Error> {
    let kind = ReviewKind::Document;
    let rows = sqlx::query_as::<_, SuggestionRow>(sql_owned(format!("{} WHERE document_id = $1 ORDER BY created_at DESC", select_suggestions(kind))))
        .bind(document_id)
        .fetch_all(&state.db)
        .await?;
    Ok(to_views(state, kind, rows).await)
}

pub async fn project_suggestions(state: &AppState, project_id: &str) -> Result<Vec<SuggestionView>, sqlx::Error> {
    let kind = ReviewKind::ProjectFile;
    let statement = format!(
        "{} WHERE file_id IN (SELECT id FROM project_files WHERE project_id = $1) ORDER BY created_at DESC",
        select_suggestions(kind)
    );
    let rows = sqlx::query_as::<_, SuggestionRow>(sql_owned(statement))
        .bind(project_id)
        .fetch_all(&state.db)
        .await?;
    Ok(to_views(state, kind, rows).await)
}

fn validate_texts(original_text: &str, replacement_text: &str) -> Result<(), ApiError> {
    if original_text == replacement_text {
        return Err((StatusCode::BAD_REQUEST, "The suggestion does not change anything".to_string()));
    }
    if original_text.len() > MAX_TEXT_LENGTH || replacement_text.len() > MAX_TEXT_LENGTH {
        return Err((StatusCode::BAD_REQUEST, "The suggestion is too long".to_string()));
    }
    Ok(())
}

pub async fn create_suggestion_for(state: &AppState, target: ReviewTarget, user_id: &str, payload: CreateSuggestionRequest) -> Result<SuggestionView, ApiError> {
    if !review_access(state, &target, user_id).await?.role.can_suggest() {
        return Err((StatusCode::FORBIDDEN, "You cannot suggest changes here".to_string()));
    }
    validate_texts(&payload.original_text, &payload.replacement_text)?;

    let kind = target.kind();
    let suggestion_id = Uuid::new_v4().to_string();
    let note = payload.note.map(|note| note.trim().to_string()).filter(|note| !note.is_empty());
    let statement = format!(
        "INSERT INTO {} (id, {}, user_id, anchor_start, anchor_end, original_text, replacement_text, note, status, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        kind.suggestions_table(),
        kind.target_column()
    );
    sqlx::query(sql_owned(statement))
        .bind(&suggestion_id)
        .bind(target.id())
        .bind(user_id)
        .bind(&payload.anchor_start)
        .bind(&payload.anchor_end)
        .bind(&payload.original_text)
        .bind(&payload.replacement_text)
        .bind(note)
        .bind(PENDING)
        .bind(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string())
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    load_suggestion(state, kind, &suggestion_id).await
}

async fn replace_content(state: &AppState, kind: ReviewKind, suggestion: &SuggestionView, user_id: &str, content: SuggestionContent) -> Result<(), ApiError> {
    if suggestion.user_id != user_id {
        return Err((StatusCode::FORBIDDEN, "Only the author can change a suggestion".to_string()));
    }
    validate_texts(&content.original_text, &content.replacement_text)?;

    let statement = format!(
        "UPDATE {} SET anchor_start = $1, anchor_end = $2, original_text = $3, replacement_text = $4 WHERE id = $5 AND status = $6",
        kind.suggestions_table()
    );
    let result = sqlx::query(sql_owned(statement))
        .bind(&content.anchor_start)
        .bind(&content.anchor_end)
        .bind(&content.original_text)
        .bind(&content.replacement_text)
        .bind(&suggestion.id)
        .bind(PENDING)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    if result.rows_affected() == 0 {
        return Err(already_handled());
    }
    Ok(())
}

fn already_handled() -> ApiError {
    (StatusCode::CONFLICT, "This suggestion was already handled".to_string())
}

fn changed_since_loaded() -> ApiError {
    (StatusCode::CONFLICT, "The suggestion changed since you loaded it".to_string())
}

async fn set_status(state: &AppState, kind: ReviewKind, suggestion_id: &str, user_id: &str, status: &str, expected_replacement: Option<&str>) -> Result<(), ApiError> {
    let mut statement = format!(
        "UPDATE {} SET status = $1, resolved_by = $2, resolved_at = $3 WHERE id = $4 AND status = $5",
        kind.suggestions_table()
    );
    if expected_replacement.is_some() {
        statement.push_str(" AND replacement_text = $6");
    }
    let mut query = sqlx::query(sql_owned(statement))
        .bind(status)
        .bind(user_id)
        .bind(timestamp_now())
        .bind(suggestion_id)
        .bind(PENDING);
    if let Some(expected) = expected_replacement {
        query = query.bind(expected);
    }
    let result = query.execute(&state.db).await.map_err(internal_error)?;
    if result.rows_affected() > 0 {
        return Ok(());
    }
    let latest = load_suggestion(state, kind, suggestion_id).await?;
    Err(if latest.status == PENDING { changed_since_loaded() } else { already_handled() })
}

pub async fn change_suggestion(state: &AppState, kind: ReviewKind, suggestion_id: &str, user_id: &str, payload: UpdateSuggestionRequest) -> Result<SuggestionView, ApiError> {
    let suggestion = load_suggestion(state, kind, suggestion_id).await?;
    let review = review_access(state, &suggestion.target, user_id).await?;

    if suggestion.status != PENDING {
        return Err(already_handled());
    }

    if let Some(content) = payload.content {
        replace_content(state, kind, &suggestion, user_id, content).await?;
    }

    let Some(status) = payload.status else {
        return load_suggestion(state, kind, suggestion_id).await;
    };

    let allowed = match status.as_str() {
        ACCEPTED | REJECTED => review.role.can_edit(),
        WITHDRAWN => suggestion.user_id == user_id,
        _ => return Err((StatusCode::BAD_REQUEST, "Status must be accepted, rejected or withdrawn".to_string())),
    };
    if !allowed {
        return Err(access::forbidden());
    }

    let expected_replacement = payload.expected_replacement_text.as_deref().filter(|_| status == ACCEPTED);
    set_status(state, kind, suggestion_id, user_id, &status, expected_replacement).await?;

    if status == ACCEPTED {
        crate::drive::activity::note_live_edit(&review.live_room, &suggestion.user_id);
    }

    load_suggestion(state, kind, suggestion_id).await
}

pub async fn list_suggestions(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<Json<Vec<SuggestionView>>, ApiError> {
    review_access(&state, &ReviewTarget::Document { document_id: document_id.clone() }, &user_id).await?;
    Ok(Json(document_suggestions(&state, &document_id).await.map_err(internal_error)?))
}

pub async fn create_suggestion(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<CreateSuggestionRequest>,
) -> Result<Json<SuggestionView>, ApiError> {
    let target = ReviewTarget::Document { document_id };
    Ok(Json(create_suggestion_for(&state, target, &user_id, payload).await?))
}

pub async fn update_suggestion(
    State(state): State<AppState>,
    Path(suggestion_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<UpdateSuggestionRequest>,
) -> Result<Json<SuggestionView>, ApiError> {
    Ok(Json(change_suggestion(&state, ReviewKind::Document, &suggestion_id, &user_id, payload).await?))
}

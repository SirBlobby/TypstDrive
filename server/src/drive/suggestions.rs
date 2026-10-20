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

const MAX_TEXT_LENGTH: usize = 20_000;
const PENDING: &str = "pending";
const ACCEPTED: &str = "accepted";
const REJECTED: &str = "rejected";
const WITHDRAWN: &str = "withdrawn";

#[derive(Serialize)]
pub struct SuggestionView {
    pub id: String,
    pub document_id: String,
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
    pub status: String,
}

type SuggestionRow = (String, String, String, String, String, String, String, Option<String>, String, Option<String>, Option<String>, String);

const SUGGESTION_COLUMNS: &str = "SELECT id, document_id, user_id, anchor_start, anchor_end, original_text, replacement_text, note, status, resolved_by, resolved_at, created_at FROM document_suggestions";

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn not_found() -> ApiError {
    (StatusCode::NOT_FOUND, "Suggestion not found".to_string())
}

async fn document_access(state: &AppState, document_id: &str, user_id: &str) -> Result<AccessRole, ApiError> {
    access::document_role(state, document_id, &Some(user_id.to_string()))
        .await
        .map(|(_, role)| role)
        .ok_or_else(access::forbidden)
}

async fn to_views(state: &AppState, rows: Vec<SuggestionRow>) -> Vec<SuggestionView> {
    let mut ids: Vec<String> = rows.iter().map(|row| row.2.clone()).collect();
    ids.extend(rows.iter().filter_map(|row| row.9.clone()));
    let directory = load_people(state, &ids).await;
    rows.into_iter()
        .map(|(id, document_id, user_id, anchor_start, anchor_end, original_text, replacement_text, note, status, resolved_by, resolved_at, created_at)| SuggestionView {
            author: directory.get(&user_id).cloned(),
            resolved_by: resolved_by.and_then(|resolver_id| directory.get(&resolver_id).cloned()),
            id,
            document_id,
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

async fn load_suggestion(state: &AppState, suggestion_id: &str) -> Result<SuggestionView, ApiError> {
    let row = sqlx::query_as::<_, SuggestionRow>(sql_owned(format!("{SUGGESTION_COLUMNS} WHERE id = $1")))
        .bind(suggestion_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .ok_or_else(not_found)?;
    to_views(state, vec![row]).await.pop().ok_or_else(not_found)
}

pub async fn list_suggestions(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<Json<Vec<SuggestionView>>, ApiError> {
    document_access(&state, &document_id, &user_id).await?;
    let rows = sqlx::query_as::<_, SuggestionRow>(sql_owned(format!(
        "{SUGGESTION_COLUMNS} WHERE document_id = $1 ORDER BY created_at DESC"
    )))
    .bind(&document_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(Json(to_views(&state, rows).await))
}

pub async fn create_suggestion(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<CreateSuggestionRequest>,
) -> Result<Json<SuggestionView>, ApiError> {
    if !document_access(&state, &document_id, &user_id).await?.can_suggest() {
        return Err((StatusCode::FORBIDDEN, "You cannot suggest changes to this document".to_string()));
    }
    if payload.original_text == payload.replacement_text {
        return Err((StatusCode::BAD_REQUEST, "The suggestion does not change anything".to_string()));
    }
    if payload.original_text.len() > MAX_TEXT_LENGTH || payload.replacement_text.len() > MAX_TEXT_LENGTH {
        return Err((StatusCode::BAD_REQUEST, "The suggestion is too long".to_string()));
    }

    let suggestion_id = Uuid::new_v4().to_string();
    let note = payload.note.map(|note| note.trim().to_string()).filter(|note| !note.is_empty());
    sqlx::query(sql(
        "INSERT INTO document_suggestions (id, document_id, user_id, anchor_start, anchor_end, original_text, replacement_text, note, status, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    ))
    .bind(&suggestion_id)
    .bind(&document_id)
    .bind(&user_id)
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

    Ok(Json(load_suggestion(&state, &suggestion_id).await?))
}

pub async fn update_suggestion(
    State(state): State<AppState>,
    Path(suggestion_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<UpdateSuggestionRequest>,
) -> Result<Json<SuggestionView>, ApiError> {
    let suggestion = load_suggestion(&state, &suggestion_id).await?;
    let role = document_access(&state, &suggestion.document_id, &user_id).await?;

    if suggestion.status != PENDING {
        return Err((StatusCode::CONFLICT, "This suggestion was already handled".to_string()));
    }

    let allowed = match payload.status.as_str() {
        ACCEPTED | REJECTED => role.can_edit(),
        WITHDRAWN => suggestion.user_id == user_id,
        _ => return Err((StatusCode::BAD_REQUEST, "Status must be accepted, rejected or withdrawn".to_string())),
    };
    if !allowed {
        return Err(access::forbidden());
    }

    sqlx::query(sql("UPDATE document_suggestions SET status = $1, resolved_by = $2, resolved_at = $3 WHERE id = $4"))
        .bind(&payload.status)
        .bind(&user_id)
        .bind(timestamp_now())
        .bind(&suggestion_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    if payload.status == ACCEPTED {
        crate::drive::activity::note_live_edit(&crate::realtime::live_documents::document_room(&suggestion.document_id), &suggestion.user_id);
    }

    Ok(Json(load_suggestion(&state, &suggestion_id).await?))
}

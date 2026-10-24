use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::format::{render_library, BibliographyFormat};
use super::targets::{clean_file_name, editable_target, write_text, Target};
use super::zotero_account::require_credentials;
use super::{internal_error, zotero, ApiError};
use crate::accounts::signed_in::SignedInUser;
use crate::db::{sql, timestamp_now};
use crate::AppState;

const MAX_LABEL_LENGTH: usize = 200;

type SyncRow = (String, Option<String>, Option<String>, String, String, String, Option<String>, Option<String>, String, i64, Option<String>, Option<String>);

macro_rules! sync_query {
    ($tail:literal) => {
        concat!(
            "SELECT s.id, s.document_id, s.project_id, s.created_by, s.library_id, s.library_name, s.collection_key, s.collection_name, \
             s.file_name, s.item_count, s.last_synced_at, u.username \
             FROM citation_syncs s LEFT JOIN users u ON u.id = s.created_by ",
            $tail
        )
    };
}

#[derive(Serialize)]
pub struct SyncView {
    pub id: String,
    pub library_name: String,
    pub collection_name: Option<String>,
    pub file_name: String,
    pub item_count: i64,
    pub last_synced_at: Option<String>,
    pub created_by_name: Option<String>,
    pub is_mine: bool,
}

#[derive(Deserialize)]
pub struct CreateSyncRequest {
    pub library_id: String,
    pub library_name: String,
    pub collection_key: Option<String>,
    pub collection_name: Option<String>,
    pub file_name: String,
}

struct SyncRecord {
    id: String,
    kind: &'static str,
    item_id: String,
    created_by: String,
    library_id: String,
    collection_key: Option<String>,
    file_name: String,
    view: SyncView,
}

fn record_from_row(row: SyncRow, viewer_id: &str) -> SyncRecord {
    let (id, document_id, project_id, created_by, library_id, library_name, collection_key, collection_name, file_name, item_count, last_synced_at, username) = row;
    let (kind, item_id) = match (document_id, project_id) {
        (Some(document_id), _) => ("document", document_id),
        (None, project_id) => ("project", project_id.unwrap_or_default()),
    };
    SyncRecord {
        view: SyncView {
            id: id.clone(),
            library_name,
            collection_name,
            file_name: file_name.clone(),
            item_count,
            last_synced_at,
            created_by_name: username,
            is_mine: created_by == viewer_id,
        },
        id,
        kind,
        item_id,
        created_by,
        library_id,
        collection_key,
        file_name,
    }
}

fn label(value: &str) -> String {
    value.trim().chars().take(MAX_LABEL_LENGTH).collect()
}

async fn load_record(state: &AppState, sync_id: &str, viewer_id: &str) -> Result<SyncRecord, ApiError> {
    sqlx::query_as::<_, SyncRow>(sql(sync_query!("WHERE s.id = $1")))
        .bind(sync_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .map(|row| record_from_row(row, viewer_id))
        .ok_or((StatusCode::NOT_FOUND, "Sync not found".to_string()))
}

fn generated_notice(format: BibliographyFormat) -> &'static str {
    match format {
        BibliographyFormat::Bibtex => "% Synced from Zotero by TypstDrive. Changes made here are replaced on the next sync.\n\n",
        BibliographyFormat::Hayagriva => "# Synced from Zotero by TypstDrive. Changes made here are replaced on the next sync.\n\n",
    }
}

async fn run(state: &AppState, record: &mut SyncRecord, target: &Target, acting_user_id: &str) -> Result<(), ApiError> {
    let account = require_credentials(state, &record.created_by).await.map_err(|(status, message)| match status {
        StatusCode::PRECONDITION_FAILED if record.created_by != acting_user_id => {
            (status, "The person who set up this sync has disconnected Zotero".to_string())
        }
        _ => (status, message),
    })?;
    let format = BibliographyFormat::for_file(&record.file_name)?;
    let library = zotero::items(&record.library_id, record.collection_key.as_deref(), &account.api_key).await?;
    let (body, count) = render_library(format, &library.items);
    let text = format!("{}{body}", generated_notice(format));
    write_text(state, target, &record.file_name, &text, acting_user_id).await?;

    let synced_at = timestamp_now();
    sqlx::query(sql("UPDATE citation_syncs SET item_count = $1, last_synced_at = $2 WHERE id = $3"))
        .bind(count as i64)
        .bind(&synced_at)
        .bind(&record.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    record.view.item_count = count as i64;
    record.view.last_synced_at = Some(synced_at);
    Ok(())
}

pub async fn list_syncs(
    State(state): State<AppState>,
    Path((kind, item_id)): Path<(String, String)>,
    SignedInUser(user_id): SignedInUser,
) -> Result<Json<Vec<SyncView>>, ApiError> {
    let target = editable_target(&state, &kind, &item_id, &Some(user_id.clone())).await?;
    let query = match target {
        Target::Document(_) => sql(sync_query!("WHERE s.document_id = $1 ORDER BY s.created_at ASC")),
        Target::Project(_) => sql(sync_query!("WHERE s.project_id = $1 ORDER BY s.created_at ASC")),
    };
    let rows = sqlx::query_as::<_, SyncRow>(query)
        .bind(target.id())
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(Json(rows.into_iter().map(|row| record_from_row(row, &user_id).view).collect()))
}

pub async fn create_sync(
    State(state): State<AppState>,
    Path((kind, item_id)): Path<(String, String)>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<CreateSyncRequest>,
) -> Result<Json<SyncView>, ApiError> {
    let target = editable_target(&state, &kind, &item_id, &Some(user_id.clone())).await?;
    require_credentials(&state, &user_id).await?;
    let file_name = clean_file_name(&target, &payload.file_name)?;
    zotero::library_prefix(&payload.library_id)?;
    let collection_key = payload.collection_key.filter(|key| !key.trim().is_empty());
    if let Some(key) = &collection_key {
        zotero::check_collection_key(key)?;
    }

    let (document_id, project_id) = match &target {
        Target::Document(document) => (Some(document.id.clone()), None),
        Target::Project(project) => (None, Some(project.id.clone())),
    };
    let sync_id = Uuid::new_v4().to_string();
    sqlx::query(sql(
        "INSERT INTO citation_syncs (id, document_id, project_id, created_by, library_id, library_name, collection_key, collection_name, file_name) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    ))
    .bind(&sync_id)
    .bind(document_id)
    .bind(project_id)
    .bind(&user_id)
    .bind(&payload.library_id)
    .bind(label(&payload.library_name))
    .bind(&collection_key)
    .bind(collection_key.as_ref().and(payload.collection_name.as_deref()).map(label))
    .bind(&file_name)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::CONFLICT, format!("{file_name} is already synced from Zotero")))?;

    let mut record = load_record(&state, &sync_id, &user_id).await?;
    if let Err(error) = run(&state, &mut record, &target, &user_id).await {
        let _ = sqlx::query(sql("DELETE FROM citation_syncs WHERE id = $1")).bind(&sync_id).execute(&state.db).await;
        return Err(error);
    }
    Ok(Json(record.view))
}

pub async fn run_sync(
    State(state): State<AppState>,
    Path(sync_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<Json<SyncView>, ApiError> {
    let mut record = load_record(&state, &sync_id, &user_id).await?;
    let target = editable_target(&state, record.kind, &record.item_id, &Some(user_id.clone())).await?;
    run(&state, &mut record, &target, &user_id).await?;
    Ok(Json(record.view))
}

pub async fn delete_sync(
    State(state): State<AppState>,
    Path(sync_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<StatusCode, ApiError> {
    let record = load_record(&state, &sync_id, &user_id).await?;
    editable_target(&state, record.kind, &record.item_id, &Some(user_id)).await?;
    sqlx::query(sql("DELETE FROM citation_syncs WHERE id = $1"))
        .bind(&record.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

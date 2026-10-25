use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use chrono::NaiveDateTime;
use serde::Serialize;

use super::{internal_error, marking, policy, purge, ApiError, TrashKind};
use crate::db::sql;
use crate::AppState;

const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

type EntryRow = (String, String, String, String, Option<String>, String, Option<String>);

#[derive(Serialize)]
pub struct TrashEntry {
    pub id: String,
    pub kind: String,
    pub item_id: String,
    pub name: String,
    pub original_folder_name: Option<String>,
    pub deleted_at: String,
    pub deleted_by_name: Option<String>,
    pub expires_at: Option<String>,
}

#[derive(Serialize)]
pub struct TrashListing {
    pub retention_days: i64,
    pub items: Vec<TrashEntry>,
}

#[derive(Serialize)]
pub struct EmptiedTrash {
    pub deleted: usize,
}

fn signed_in_account(jar: &SignedCookieJar) -> Result<String, ApiError> {
    jar.get("session_user_id")
        .map(|cookie| cookie.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))
}

fn expiry(deleted_at: &str, retention_days: i64) -> Option<String> {
    let deleted = NaiveDateTime::parse_from_str(deleted_at, TIMESTAMP_FORMAT).ok()?;
    Some((deleted + chrono::Duration::days(retention_days)).format(TIMESTAMP_FORMAT).to_string())
}

async fn owned_entry(state: &AppState, entry_id: &str, owner_id: &str) -> Result<(TrashKind, String), ApiError> {
    let (kind, item_id) = sqlx::query_as::<_, (String, String)>(sql("SELECT kind, item_id FROM trash_items WHERE id = $1 AND owner_id = $2"))
        .bind(entry_id)
        .bind(owner_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "This item is no longer in the trash".to_string()))?;
    let kind = TrashKind::parse(&kind).ok_or((StatusCode::NOT_FOUND, "This item is no longer in the trash".to_string()))?;
    Ok((kind, item_id))
}

pub async fn list_trash(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<TrashListing>, ApiError> {
    let owner_id = signed_in_account(&jar)?;
    purge::remove_orphans(&state).await;
    let retention_days = policy::retention_days(&state).await;

    let rows = sqlx::query_as::<_, EntryRow>(sql(
        "SELECT t.id, t.kind, t.item_id, t.name, f.name, t.deleted_at, u.username FROM trash_items t \
         LEFT JOIN folders f ON f.id = t.original_folder_id AND f.deleted_at IS NULL \
         LEFT JOIN users u ON u.id = t.deleted_by \
         WHERE t.owner_id = $1 ORDER BY t.deleted_at DESC",
    ))
    .bind(&owner_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let items = rows
        .into_iter()
        .map(|(id, kind, item_id, name, original_folder_name, deleted_at, deleted_by_name)| TrashEntry {
            expires_at: expiry(&deleted_at, retention_days),
            id,
            kind,
            item_id,
            name,
            original_folder_name,
            deleted_at,
            deleted_by_name,
        })
        .collect();
    Ok(Json(TrashListing { retention_days, items }))
}

pub async fn restore_entry(
    State(state): State<AppState>,
    Path(entry_id): Path<String>,
    jar: SignedCookieJar,
) -> Result<StatusCode, ApiError> {
    let owner_id = signed_in_account(&jar)?;
    let (kind, item_id) = owned_entry(&state, &entry_id, &owner_id).await?;
    marking::restore_rows(&state, &entry_id, kind, &item_id).await?;

    let resource = match kind {
        TrashKind::Document => Some(crate::sharing::access::Resource::Document(item_id)),
        TrashKind::Project => Some(crate::sharing::access::Resource::Project(item_id)),
        _ => None,
    };
    if let Some(resource) = resource {
        crate::sharing::access::revalidate_resource(&state, resource).await;
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_entry(
    State(state): State<AppState>,
    Path(entry_id): Path<String>,
    jar: SignedCookieJar,
) -> Result<StatusCode, ApiError> {
    let owner_id = signed_in_account(&jar)?;
    let (kind, item_id) = owned_entry(&state, &entry_id, &owner_id).await?;
    purge::purge_entry(&state, &entry_id, kind, &item_id).await?;
    purge::remove_orphans(&state).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn empty_trash(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<EmptiedTrash>, ApiError> {
    let owner_id = signed_in_account(&jar)?;
    let entries = sqlx::query_as::<_, (String, String, String)>(sql("SELECT id, kind, item_id FROM trash_items WHERE owner_id = $1"))
        .bind(&owner_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
    let deleted = purge::purge_entries(&state, entries).await;
    Ok(Json(EmptiedTrash { deleted }))
}

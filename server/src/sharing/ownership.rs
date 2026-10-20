use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};
use sqlx::{AssertSqlSafe, FromRow};
use uuid::Uuid;

use crate::{
    desktop::devices::{notify_devices, DeviceEvent},
    sharing::ownership_moves::{item_size, transfer_item},
    sharing::members::{owner_of, revalidate, ShareKind},
    AppState,
};

type ApiError = (StatusCode, String);

#[derive(Deserialize)]
pub struct TransferRequest {
    pub item_kind: String,
    pub item_id: String,
    pub to_user_id: String,
}

#[derive(Serialize, FromRow)]
pub struct TransferView {
    pub id: String,
    pub item_kind: String,
    pub item_id: String,
    pub from_user_id: String,
    pub from_username: String,
    pub to_user_id: String,
    pub to_username: String,
    pub created_at: String,
    #[sqlx(default)]
    pub item_name: Option<String>,
}

#[derive(Serialize)]
pub struct TransferList {
    pub incoming: Vec<TransferView>,
    pub outgoing: Vec<TransferView>,
}

#[derive(Serialize)]
pub struct ShareInfo {
    pub is_owner: bool,
    pub pending_transfer: Option<TransferView>,
}

const TRANSFER_COLUMNS: &str = "SELECT t.id, t.item_kind, t.item_id, t.from_user_id, sender.username AS from_username, \
     t.to_user_id, receiver.username AS to_username, t.created_at \
     FROM ownership_transfers t \
     INNER JOIN users sender ON sender.id = t.from_user_id \
     INNER JOIN users receiver ON receiver.id = t.to_user_id";

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn session_user(jar: &SignedCookieJar) -> Result<String, ApiError> {
    jar.get("session_user_id")
        .map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))
}

fn parse_kind(value: &str) -> Result<ShareKind, ApiError> {
    ShareKind::parse(value).ok_or((StatusCode::BAD_REQUEST, "Unknown item type".to_string()))
}

fn name_column(kind: ShareKind) -> &'static str {
    match kind {
        ShareKind::Document => "title",
        ShareKind::Project | ShareKind::Folder => "name",
    }
}

async fn is_direct_member(state: &AppState, kind: ShareKind, item_id: &str, user_id: &str) -> Result<bool, ApiError> {
    let sql = format!(
        "SELECT CAST(COUNT(*) AS BIGINT) FROM {} WHERE {} = $1 AND user_id = $2",
        kind.collaborator_table(),
        kind.item_column()
    );
    let (count,) = sqlx::query_as::<_, (i64,)>(AssertSqlSafe(sql))
        .bind(item_id)
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(count > 0)
}

async fn current_name_and_owner(state: &AppState, kind: ShareKind, item_id: &str) -> Option<(String, String)> {
    let sql = format!("SELECT {}, owner_id FROM {} WHERE id = $1", name_column(kind), kind.item_table());
    sqlx::query_as::<_, (String, String)>(AssertSqlSafe(sql))
        .bind(item_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
}

async fn still_valid(state: &AppState, transfer: &mut TransferView) -> bool {
    let Some(kind) = ShareKind::parse(&transfer.item_kind) else {
        return false;
    };
    match current_name_and_owner(state, kind, &transfer.item_id).await {
        Some((name, owner_id)) if owner_id == transfer.from_user_id => {
            transfer.item_name = Some(name);
            is_direct_member(state, kind, &transfer.item_id, &transfer.to_user_id).await.unwrap_or(false)
        }
        _ => false,
    }
}

async fn fetch_transfers(state: &AppState, condition: &'static str, value: &str) -> Result<Vec<TransferView>, ApiError> {
    let sql = format!("{TRANSFER_COLUMNS} WHERE {condition} ORDER BY t.created_at DESC");
    let transfers = sqlx::query_as::<_, TransferView>(AssertSqlSafe(sql))
        .bind(value)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;

    let mut valid = Vec::new();
    for mut transfer in transfers {
        if still_valid(state, &mut transfer).await {
            valid.push(transfer);
        } else {
            remove_transfer(state, &transfer.id).await?;
        }
    }
    Ok(valid)
}

async fn remove_transfer(state: &AppState, transfer_id: &str) -> Result<(), ApiError> {
    sqlx::query("DELETE FROM ownership_transfers WHERE id = $1")
        .bind(transfer_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(())
}

pub async fn request_transfer(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<TransferRequest>,
) -> Result<Json<TransferView>, ApiError> {
    let user_id = session_user(&jar)?;
    let kind = parse_kind(&payload.item_kind)?;

    if owner_of(&state, kind, &payload.item_id).await? != user_id {
        return Err((StatusCode::FORBIDDEN, format!("Only the owner can transfer this {}", kind.label())));
    }
    if payload.to_user_id == user_id {
        return Err((StatusCode::BAD_REQUEST, format!("You already own this {}", kind.label())));
    }
    if !is_direct_member(&state, kind, &payload.item_id, &payload.to_user_id).await? {
        return Err((StatusCode::BAD_REQUEST, format!("Ownership can only go to someone this {} is shared with", kind.label())));
    }

    sqlx::query("DELETE FROM ownership_transfers WHERE item_kind = $1 AND item_id = $2")
        .bind(kind.label())
        .bind(&payload.item_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    let transfer_id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO ownership_transfers (id, item_kind, item_id, from_user_id, to_user_id) VALUES ($1, $2, $3, $4, $5)")
        .bind(&transfer_id)
        .bind(kind.label())
        .bind(&payload.item_id)
        .bind(&user_id)
        .bind(&payload.to_user_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    fetch_transfers(&state, "t.id = $1", &transfer_id)
        .await?
        .pop()
        .map(Json)
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Transfer could not be created".to_string()))
}

pub async fn list_transfers(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<TransferList>, ApiError> {
    let user_id = session_user(&jar)?;
    Ok(Json(TransferList {
        incoming: fetch_transfers(&state, "t.to_user_id = $1", &user_id).await?,
        outgoing: fetch_transfers(&state, "t.from_user_id = $1", &user_id).await?,
    }))
}

pub async fn accept_transfer(
    State(state): State<AppState>,
    Path(transfer_id): Path<String>,
    jar: SignedCookieJar,
) -> Result<StatusCode, ApiError> {
    let user_id = session_user(&jar)?;
    let transfer = fetch_transfers(&state, "t.id = $1", &transfer_id)
        .await?
        .pop()
        .ok_or((StatusCode::NOT_FOUND, "This transfer is no longer available".to_string()))?;

    if transfer.to_user_id != user_id {
        return Err((StatusCode::FORBIDDEN, "This transfer was not sent to you".to_string()));
    }

    let kind = parse_kind(&transfer.item_kind)?;
    let size = item_size(&state, kind, &transfer.item_id, &transfer.from_user_id).await?;
    crate::accounts::storage::ensure_room(&state, &user_id, size).await?;

    transfer_item(&state, kind, &transfer.item_id, &transfer.from_user_id, &user_id).await?;

    revalidate(&state, kind, &transfer.item_id, &user_id).await;
    notify_devices(&state, &user_id, DeviceEvent::structure()).await;
    notify_devices(&state, &transfer.from_user_id, DeviceEvent::structure()).await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn cancel_transfer(
    State(state): State<AppState>,
    Path(transfer_id): Path<String>,
    jar: SignedCookieJar,
) -> Result<StatusCode, ApiError> {
    let user_id = session_user(&jar)?;
    let result = sqlx::query("DELETE FROM ownership_transfers WHERE id = $1 AND (from_user_id = $2 OR to_user_id = $2)")
        .bind(&transfer_id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Transfer not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn share_info(
    State(state): State<AppState>,
    Path((item_kind, item_id)): Path<(String, String)>,
    jar: SignedCookieJar,
) -> Result<Json<ShareInfo>, ApiError> {
    let user_id = session_user(&jar)?;
    let kind = parse_kind(&item_kind)?;
    let is_owner = owner_of(&state, kind, &item_id).await? == user_id;

    let pending_transfer = if is_owner {
        fetch_transfers(&state, "t.item_id = $1", &item_id)
            .await?
            .into_iter()
            .find(|transfer| transfer.item_kind == kind.label())
    } else {
        None
    };

    Ok(Json(ShareInfo { is_owner, pending_transfer }))
}

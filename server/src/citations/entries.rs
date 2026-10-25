use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use super::csl::CslItem;
use super::format::{render_entry, BibliographyFormat};
use super::identifiers::parse_identifier;
use super::sources::fetch_csl;
use super::policy::load_policy;
use super::targets::{bibliography_files, clean_file_name, editable_target, read_text, write_text};
use super::ApiError;
use crate::accounts::signed_in::SignedInUser;
use crate::AppState;

const LOOKUPS_PER_MINUTE: u32 = 30;

#[derive(Deserialize)]
pub struct LookupRequest {
    pub identifier: String,
    pub format: String,
}

#[derive(Serialize)]
pub struct LookupResult {
    pub key: String,
    pub title: String,
    pub format: &'static str,
    pub entry: String,
}

#[derive(Deserialize)]
pub struct AddEntryRequest {
    pub identifier: String,
    pub file_name: String,
}

#[derive(Serialize)]
pub struct AddedEntry {
    pub key: String,
    pub title: String,
    pub file_name: String,
    pub entry: String,
    pub created_file: bool,
}

pub(super) fn check_rate(user_id: &str) -> Result<(), ApiError> {
    static WINDOWS: OnceLock<Mutex<HashMap<String, (u32, Instant)>>> = OnceLock::new();
    let mut windows = WINDOWS.get_or_init(|| Mutex::new(HashMap::new())).lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Lookup limiter failed".to_string()))?;
    let now = Instant::now();
    windows.retain(|_, (_, started)| now.duration_since(*started) < Duration::from_secs(60));
    let window = windows.entry(user_id.to_string()).or_insert((0, now));
    if window.0 >= LOOKUPS_PER_MINUTE {
        return Err((StatusCode::TOO_MANY_REQUESTS, "Too many lookups. Wait a minute and try again.".to_string()));
    }
    window.0 += 1;
    Ok(())
}

async fn look_up(state: &AppState, user_id: &str, identifier: &str) -> Result<serde_json::Value, ApiError> {
    load_policy(state).await.check_lookup()?;
    let identifier = parse_identifier(identifier)?;
    check_rate(user_id)?;
    fetch_csl(&identifier).await
}

fn title_of(item: &serde_json::Value) -> String {
    CslItem::new(item).text("title").unwrap_or_else(|| "Untitled".to_string())
}

pub async fn lookup(
    State(state): State<AppState>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<LookupRequest>,
) -> Result<Json<LookupResult>, ApiError> {
    let format = BibliographyFormat::parse(&payload.format)?;
    let item = look_up(&state, &user_id, &payload.identifier).await?;
    let rendered = render_entry(format, &item, &mut HashSet::new());
    Ok(Json(LookupResult { key: rendered.key, title: title_of(&item), format: format.as_str(), entry: rendered.text }))
}

pub async fn list_files(
    State(state): State<AppState>,
    Path((kind, item_id)): Path<(String, String)>,
    SignedInUser(user_id): SignedInUser,
) -> Result<Json<Vec<String>>, ApiError> {
    let target = editable_target(&state, &kind, &item_id, &Some(user_id)).await?;
    Ok(Json(bibliography_files(&state, &target).await?))
}

pub async fn add_entry(
    State(state): State<AppState>,
    Path((kind, item_id)): Path<(String, String)>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<AddEntryRequest>,
) -> Result<Json<AddedEntry>, ApiError> {
    let target = editable_target(&state, &kind, &item_id, &Some(user_id.clone())).await?;
    let file_name = clean_file_name(&target, &payload.file_name)?;
    let format = BibliographyFormat::for_file(&file_name)?;
    let item = look_up(&state, &user_id, &payload.identifier).await?;

    let existing = read_text(&state, &target, &file_name).await?;
    let current = existing.clone().unwrap_or_default();
    let mut taken = format.existing_keys(&current);
    let rendered = render_entry(format, &item, &mut taken);

    let trimmed = current.trim_end();
    let updated = if trimmed.is_empty() { rendered.text.clone() } else { format!("{trimmed}\n\n{}", rendered.text) };
    write_text(&state, &target, &file_name, &updated, &user_id).await?;

    Ok(Json(AddedEntry {
        key: rendered.key,
        title: title_of(&item),
        file_name,
        entry: rendered.text,
        created_file: existing.is_none(),
    }))
}

use std::time::Duration;

use axum::http::StatusCode;
use serde::Serialize;
use serde_json::Value;

use super::ApiError;

const API_BASE: &str = "https://api.zotero.org";
const PAGE_SIZE: usize = 100;
const MAX_ITEMS: usize = 5000;
const REQUEST_TIMEOUT_SECONDS: u64 = 20;
const SKIPPED_ITEM_TYPES: &[&str] = &["attachment", "note", "annotation"];

pub struct KeyInfo {
    pub user_id: String,
    pub username: String,
}

#[derive(Serialize)]
pub struct Library {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct Collection {
    pub key: String,
    pub name: String,
    pub parent_key: Option<String>,
}

pub struct LibraryItems {
    pub items: Vec<Value>,
}

struct Page {
    body: Value,
    total: Option<usize>,
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .user_agent("TypstDrive (Zotero sync)")
        .build()
}

fn bad_gateway(message: &str) -> ApiError {
    (StatusCode::BAD_GATEWAY, message.to_string())
}

fn is_safe_segment(value: &str) -> bool {
    !value.is_empty() && value.len() <= 64 && value.chars().all(|c| c.is_ascii_alphanumeric())
}

pub fn library_prefix(library_id: &str) -> Result<String, ApiError> {
    let invalid = || (StatusCode::BAD_REQUEST, "Unknown Zotero library".to_string());
    let (kind, id) = library_id.split_once(':').ok_or_else(invalid)?;
    if !is_safe_segment(id) {
        return Err(invalid());
    }
    match kind {
        "user" => Ok(format!("users/{id}")),
        "group" => Ok(format!("groups/{id}")),
        _ => Err(invalid()),
    }
}

pub fn check_collection_key(key: &str) -> Result<(), ApiError> {
    if is_safe_segment(key) {
        Ok(())
    } else {
        Err((StatusCode::BAD_REQUEST, "Unknown Zotero collection".to_string()))
    }
}

async fn get(path: String, api_key: String) -> Result<Page, ApiError> {
    tokio::task::spawn_blocking(move || {
        let url = format!("{API_BASE}/{path}");
        let response = agent()
            .get(&url)
            .set("Zotero-API-Version", "3")
            .set("Zotero-API-Key", &api_key)
            .call();
        let response = match response {
            Ok(response) => response,
            Err(ureq::Error::Status(403, _)) => return Err((StatusCode::FORBIDDEN, "Zotero refused the API key. Reconnect Zotero in your settings.".to_string())),
            Err(ureq::Error::Status(404, _)) => return Err((StatusCode::NOT_FOUND, "That Zotero library or collection no longer exists".to_string())),
            Err(ureq::Error::Status(429, _)) => return Err((StatusCode::TOO_MANY_REQUESTS, "Zotero is rate limiting requests. Try again in a minute.".to_string())),
            Err(ureq::Error::Status(code, _)) => return Err(bad_gateway(&format!("Zotero answered with status {code}"))),
            Err(_) => return Err(bad_gateway("Could not reach Zotero")),
        };
        let total = response.header("Total-Results").and_then(|value| value.parse().ok());
        let body = response.into_json::<Value>().map_err(|_| bad_gateway("Zotero sent an unreadable answer"))?;
        Ok(Page { body, total })
    })
    .await
    .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
}

pub async fn key_info(api_key: &str) -> Result<KeyInfo, ApiError> {
    if !is_safe_segment(api_key) {
        return Err((StatusCode::BAD_REQUEST, "That does not look like a Zotero API key".to_string()));
    }
    let page = get(format!("keys/{api_key}"), api_key.to_string()).await.map_err(|(status, message)| match status {
        StatusCode::FORBIDDEN | StatusCode::NOT_FOUND => (StatusCode::BAD_REQUEST, "Zotero does not recognise that API key".to_string()),
        _ => (status, message),
    })?;
    let user_id = page.body.get("userID").map(|id| id.to_string().trim_matches('"').to_string()).filter(|id| is_safe_segment(id));
    let username = page.body.get("username").and_then(Value::as_str).unwrap_or_default().to_string();
    let user_id = user_id.ok_or_else(|| bad_gateway("Zotero did not say which account the key belongs to"))?;
    Ok(KeyInfo { user_id, username })
}

async fn all_pages(path: &str, query: &str, api_key: &str) -> Result<Vec<Value>, ApiError> {
    let mut results = Vec::new();
    let mut start = 0;
    loop {
        let separator = if query.is_empty() { "" } else { "&" };
        let page = get(format!("{path}?{query}{separator}limit={PAGE_SIZE}&start={start}"), api_key.to_string()).await?;
        let entries = page.body.as_array().cloned().unwrap_or_default();
        let fetched = entries.len();
        results.extend(entries);
        start += fetched;
        let finished = fetched < PAGE_SIZE || page.total.is_some_and(|total| start >= total);
        if finished {
            break;
        }
        if start >= MAX_ITEMS {
            return Err((StatusCode::PAYLOAD_TOO_LARGE, format!("Only libraries with up to {MAX_ITEMS} items can be synced. Pick a collection instead.")));
        }
    }
    Ok(results)
}

pub async fn libraries(user_id: &str, username: &str, api_key: &str) -> Result<Vec<Library>, ApiError> {
    let mut libraries = vec![Library { id: format!("user:{user_id}"), name: format!("My Library ({username})") }];
    let groups = all_pages(&format!("users/{user_id}/groups"), "", api_key).await?;
    for group in groups {
        let id = group.get("id").map(|id| id.to_string().trim_matches('"').to_string()).unwrap_or_default();
        let name = group.pointer("/data/name").and_then(Value::as_str).unwrap_or("Group").to_string();
        if is_safe_segment(&id) {
            libraries.push(Library { id: format!("group:{id}"), name });
        }
    }
    Ok(libraries)
}

pub async fn collections(library_id: &str, api_key: &str) -> Result<Vec<Collection>, ApiError> {
    let prefix = library_prefix(library_id)?;
    let entries = all_pages(&format!("{prefix}/collections"), "", api_key).await?;
    let mut collections: Vec<Collection> = entries
        .into_iter()
        .filter_map(|entry| {
            let key = entry.get("key").and_then(Value::as_str)?.to_string();
            let name = entry.pointer("/data/name").and_then(Value::as_str).unwrap_or("Untitled").to_string();
            let parent_key = entry.pointer("/data/parentCollection").and_then(Value::as_str).map(str::to_string);
            Some(Collection { key, name, parent_key })
        })
        .collect();
    collections.sort_by(|first, second| first.name.to_lowercase().cmp(&second.name.to_lowercase()));
    Ok(collections)
}

pub async fn items(library_id: &str, collection_key: Option<&str>, api_key: &str) -> Result<LibraryItems, ApiError> {
    let prefix = library_prefix(library_id)?;
    let path = match collection_key {
        Some(key) => {
            check_collection_key(key)?;
            format!("{prefix}/collections/{key}/items")
        }
        None => format!("{prefix}/items"),
    };
    let entries = all_pages(&path, "include=csljson,data", api_key).await?;
    let items = entries
        .into_iter()
        .filter(|entry| {
            let item_type = entry.pointer("/data/itemType").and_then(Value::as_str).unwrap_or_default();
            !SKIPPED_ITEM_TYPES.contains(&item_type)
        })
        .filter_map(|entry| {
            let mut csl = entry.get("csljson").cloned()?;
            let citation_key = entry.pointer("/data/citationKey").and_then(Value::as_str).filter(|key| !key.is_empty());
            if let (Some(key), Some(object)) = (citation_key, csl.as_object_mut()) {
                object.insert("citation-key".to_string(), Value::String(key.to_string()));
            }
            Some(csl)
        })
        .collect();
    Ok(LibraryItems { items })
}

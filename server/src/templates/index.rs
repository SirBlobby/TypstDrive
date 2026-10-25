use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::{http_agent, unreachable_registry, ApiError, REGISTRY_URL};
use crate::accounts::signed_in::SignedInUser;

const REFRESH_AFTER: Duration = Duration::from_secs(6 * 60 * 60);
const RETRY_AFTER: Duration = Duration::from_secs(5 * 60);

#[derive(Deserialize)]
struct IndexEntry {
    name: String,
    version: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    authors: Vec<String>,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    disciplines: Vec<String>,
    #[serde(default)]
    keywords: Vec<String>,
    #[serde(default)]
    template: Option<IndexTemplate>,
    #[serde(default, rename = "updatedAt")]
    updated_at: Option<i64>,
}

#[derive(Deserialize)]
struct IndexTemplate {
    path: String,
    entrypoint: String,
    #[serde(default)]
    thumbnail: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct Template {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub categories: Vec<String>,
    pub disciplines: Vec<String>,
    pub keywords: Vec<String>,
    pub has_thumbnail: bool,
    pub updated_at: Option<i64>,
    #[serde(skip)]
    pub template_path: String,
    #[serde(skip)]
    pub entrypoint: String,
}

struct CachedIndex {
    templates: Arc<Vec<Template>>,
    fetched_at: Instant,
}

static INDEX: RwLock<Option<CachedIndex>> = RwLock::const_new(None);

fn version_parts(version: &str) -> (u64, u64, u64) {
    let mut parts = version.split('.').map(|part| part.parse::<u64>().unwrap_or(0));
    (parts.next().unwrap_or(0), parts.next().unwrap_or(0), parts.next().unwrap_or(0))
}

fn author_name(author: &str) -> String {
    match author.find('<') {
        Some(position) => author[..position].trim().to_string(),
        None => author.trim().to_string(),
    }
}

fn latest_templates(entries: Vec<IndexEntry>) -> Vec<Template> {
    let mut latest: HashMap<String, IndexEntry> = HashMap::new();
    for entry in entries {
        let is_newer = latest
            .get(&entry.name)
            .is_none_or(|current| version_parts(&entry.version) > version_parts(&current.version));
        if is_newer {
            latest.insert(entry.name.clone(), entry);
        }
    }

    let mut templates: Vec<Template> = latest
        .into_values()
        .filter_map(|entry| {
            let template = entry.template?;
            Some(Template {
                name: entry.name,
                version: entry.version,
                description: entry.description.unwrap_or_default(),
                authors: entry.authors.iter().map(|author| author_name(author)).filter(|name| !name.is_empty()).collect(),
                categories: entry.categories,
                disciplines: entry.disciplines,
                keywords: entry.keywords,
                has_thumbnail: template.thumbnail.is_some(),
                updated_at: entry.updated_at,
                template_path: template.path,
                entrypoint: template.entrypoint,
            })
        })
        .collect();
    templates.sort_by(|left, right| left.name.cmp(&right.name));
    templates
}

async fn download_index() -> Option<Vec<Template>> {
    tokio::task::spawn_blocking(|| {
        let url = format!("{REGISTRY_URL}/preview/index.json");
        match http_agent().get(&url).call() {
            Ok(response) => match response.into_json::<Vec<IndexEntry>>() {
                Ok(entries) => Some(latest_templates(entries)),
                Err(error) => {
                    tracing::warn!("Could not read the Typst Universe index: {error}");
                    None
                }
            },
            Err(error) => {
                tracing::warn!("Could not download the Typst Universe index: {error}");
                None
            }
        }
    })
    .await
    .ok()
    .flatten()
}

fn is_fresh(cached: &Option<CachedIndex>) -> bool {
    cached.as_ref().is_some_and(|index| index.fetched_at.elapsed() < REFRESH_AFTER)
}

pub async fn templates() -> Result<Arc<Vec<Template>>, ApiError> {
    {
        let cached = INDEX.read().await;
        if is_fresh(&cached) {
            if let Some(index) = cached.as_ref() {
                return Ok(index.templates.clone());
            }
        }
    }

    let mut cached = INDEX.write().await;
    if !is_fresh(&cached) {
        match download_index().await {
            Some(templates) => {
                *cached = Some(CachedIndex { templates: Arc::new(templates), fetched_at: Instant::now() });
            }
            None => {
                if let Some(index) = cached.as_mut() {
                    index.fetched_at = Instant::now().checked_sub(REFRESH_AFTER - RETRY_AFTER).unwrap_or_else(Instant::now);
                }
            }
        }
    }
    cached.as_ref().map(|index| index.templates.clone()).ok_or_else(unreachable_registry)
}

pub async fn find(name: &str, version: &str) -> Result<Template, ApiError> {
    templates()
        .await?
        .iter()
        .find(|template| template.name == name && template.version == version)
        .cloned()
        .ok_or((axum::http::StatusCode::NOT_FOUND, "Template not found".to_string()))
}

pub async fn list_templates(_: SignedInUser) -> Result<Json<Vec<Template>>, ApiError> {
    let templates = templates().await?;
    Ok(Json(templates.as_ref().clone()))
}

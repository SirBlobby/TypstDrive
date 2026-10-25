use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, LazyLock, Mutex};

use axum::extract::Path;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

use super::{http_agent, index, internal_error, unreachable_registry, ApiError, REGISTRY_URL};
use crate::accounts::signed_in::SignedInUser;

const MAX_CACHED_THUMBNAILS: usize = 600;
const MAX_THUMBNAIL_BYTES: u64 = 2 * 1024 * 1024;

type CachedThumbnail = Option<Arc<Vec<u8>>>;

static THUMBNAILS: LazyLock<Mutex<HashMap<String, CachedThumbnail>>> = LazyLock::new(Default::default);

fn cached(key: &str) -> Option<CachedThumbnail> {
    THUMBNAILS.lock().ok()?.get(key).cloned()
}

fn remember(key: String, thumbnail: CachedThumbnail) {
    if let Ok(mut thumbnails) = THUMBNAILS.lock() {
        if thumbnails.len() >= MAX_CACHED_THUMBNAILS {
            thumbnails.clear();
        }
        thumbnails.insert(key, thumbnail);
    }
}

async fn download(name: String, version: String) -> Result<CachedThumbnail, ApiError> {
    tokio::task::spawn_blocking(move || {
        let url = format!("{REGISTRY_URL}/preview/thumbnails/{name}-{version}-small.webp");
        match http_agent().get(&url).call() {
            Ok(response) => {
                let mut bytes = Vec::new();
                response
                    .into_reader()
                    .take(MAX_THUMBNAIL_BYTES)
                    .read_to_end(&mut bytes)
                    .map_err(|_| unreachable_registry())?;
                Ok(Some(Arc::new(bytes)))
            }
            Err(ureq::Error::Status(404, _)) => Ok(None),
            Err(_) => Err(unreachable_registry()),
        }
    })
    .await
    .map_err(internal_error)?
}

pub async fn thumbnail(_: SignedInUser, Path((name, version)): Path<(String, String)>) -> Result<Response, ApiError> {
    let not_found = || (StatusCode::NOT_FOUND, "No preview for this template".to_string());
    let template = index::find(&name, &version).await?;
    if !template.has_thumbnail {
        return Err(not_found());
    }

    let key = format!("{name}@{version}");
    let thumbnail = match cached(&key) {
        Some(thumbnail) => thumbnail,
        None => {
            let downloaded = download(name, version).await?;
            remember(key, downloaded.clone());
            downloaded
        }
    };
    let image = thumbnail.ok_or_else(not_found)?;

    Ok((
        [(header::CONTENT_TYPE, "image/webp"), (header::CACHE_CONTROL, "public, max-age=604800, immutable")],
        image.as_ref().clone(),
    )
        .into_response())
}

mod cache;
pub mod maintenance;

use std::sync::Arc;

use axum::http::StatusCode;
use futures_util::future::join_all;
use object_store::aws::AmazonS3Builder;
use object_store::path::Path as ObjectPath;
use object_store::{ObjectStore, PutPayload};
use uuid::Uuid;

use cache::ObjectCache;

const DEFAULT_CACHE_MEGABYTES: usize = 256;

type ApiError = (StatusCode, String);

pub struct StoredData {
    pub inline_data: Vec<u8>,
    pub storage_key: Option<String>,
    pub size_bytes: i64,
}

struct RemoteStore {
    store: Arc<dyn ObjectStore>,
    prefix: String,
}

pub struct FileStorage {
    remote: Option<RemoteStore>,
    cache: ObjectCache,
}

fn environment_flag(name: &str) -> bool {
    std::env::var(name).map(|value| value.eq_ignore_ascii_case("true") || value == "1").unwrap_or(false)
}

fn non_empty_environment(name: &str) -> Option<String> {
    std::env::var(name).ok().map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

fn storage_error(error: object_store::Error) -> ApiError {
    tracing::error!("object storage request failed: {error}");
    (StatusCode::BAD_GATEWAY, "File storage is unavailable".to_string())
}

fn build_remote(bucket: String) -> RemoteStore {
    let mut builder = AmazonS3Builder::new()
        .with_bucket_name(bucket)
        .with_region(non_empty_environment("S3_REGION").unwrap_or_else(|| "us-east-1".to_string()))
        .with_virtual_hosted_style_request(environment_flag("S3_VIRTUAL_HOSTED_STYLE"));
    if let Some(endpoint) = non_empty_environment("S3_ENDPOINT") {
        builder = builder.with_allow_http(endpoint.starts_with("http://")).with_endpoint(endpoint);
    }
    if let Some(access_key) = non_empty_environment("S3_ACCESS_KEY_ID") {
        builder = builder.with_access_key_id(access_key);
    }
    if let Some(secret_key) = non_empty_environment("S3_SECRET_ACCESS_KEY") {
        builder = builder.with_secret_access_key(secret_key);
    }

    let store = builder
        .build()
        .expect("Invalid object storage configuration. Check the S3_* environment variables.");
    let prefix = non_empty_environment("S3_PREFIX").unwrap_or_else(|| "typstdrive".to_string());
    RemoteStore {
        store: Arc::new(store),
        prefix: prefix.trim_matches('/').to_string(),
    }
}

impl FileStorage {
    pub fn from_environment() -> Self {
        let cache_megabytes = non_empty_environment("OBJECT_CACHE_MB")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(DEFAULT_CACHE_MEGABYTES);
        let remote = non_empty_environment("S3_BUCKET").map(build_remote);
        if remote.is_some() {
            tracing::info!("Uploaded files are stored in S3-compatible object storage");
        }
        Self {
            remote,
            cache: ObjectCache::new(cache_megabytes * 1024 * 1024),
        }
    }

    pub fn uses_object_storage(&self) -> bool {
        self.remote.is_some()
    }

    pub fn cache_limit_bytes(&self) -> usize {
        self.cache.limit_bytes()
    }

    fn files_prefix(remote: &RemoteStore) -> String {
        format!("{}/files", remote.prefix)
    }

    pub async fn store(&self, data: Vec<u8>) -> Result<StoredData, ApiError> {
        let size_bytes = data.len() as i64;
        let Some(remote) = &self.remote else {
            return Ok(StoredData { inline_data: data, storage_key: None, size_bytes });
        };

        let storage_key = format!("{}/{}", Self::files_prefix(remote), Uuid::new_v4());
        remote
            .store
            .put(&ObjectPath::from(storage_key.as_str()), PutPayload::from(data.clone()))
            .await
            .map_err(storage_error)?;
        self.cache.insert(&storage_key, data);
        Ok(StoredData { inline_data: Vec::new(), storage_key: Some(storage_key), size_bytes })
    }

    pub async fn load(&self, inline_data: Vec<u8>, storage_key: Option<String>) -> Result<Vec<u8>, ApiError> {
        let Some(storage_key) = storage_key else {
            return Ok(inline_data);
        };
        if let Some(cached) = self.cache.get(&storage_key) {
            return Ok(cached);
        }
        let Some(remote) = &self.remote else {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                "This file is kept in object storage, which is not configured".to_string(),
            ));
        };

        let data = remote
            .store
            .get(&ObjectPath::from(storage_key.as_str()))
            .await
            .map_err(storage_error)?
            .bytes()
            .await
            .map_err(storage_error)?
            .to_vec();
        self.cache.insert(&storage_key, data.clone());
        Ok(data)
    }

    pub async fn load_named(&self, rows: Vec<(String, Vec<u8>, Option<String>)>) -> Result<Vec<(String, Vec<u8>)>, ApiError> {
        let loads = rows.into_iter().map(|(name, inline_data, storage_key)| async move {
            self.load(inline_data, storage_key).await.map(|data| (name, data))
        });
        join_all(loads).await.into_iter().collect()
    }

    pub async fn load_named_skipping_failures(&self, rows: Vec<(String, Vec<u8>, Option<String>)>) -> Vec<(String, Vec<u8>)> {
        let loads = rows.into_iter().map(|(name, inline_data, storage_key)| async move {
            self.load(inline_data, storage_key).await.ok().map(|data| (name, data))
        });
        join_all(loads).await.into_iter().flatten().collect()
    }
}

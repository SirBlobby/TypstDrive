use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::api::render::InlineFile;
use crate::db::sql;
use crate::AppState;

const CACHE_MAX_AGE_SECONDS: i64 = 3600;
const CACHE_PRUNE_INTERVAL_SECONDS: i64 = 300;

static LAST_CACHE_PRUNE: AtomicI64 = AtomicI64::new(0);

pub fn cache_key(account_id: &str, parts: &[&str], files: &Option<Vec<InlineFile>>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    for part in parts {
        hasher.update(b"\x00");
        hasher.update(part.as_bytes());
    }
    if let Some(files) = files {
        let mut pairs: Vec<_> = files.iter().map(|file| (file.name.as_str(), file.data.as_str())).collect();
        pairs.sort_by_key(|(name, _)| *name);
        for (name, data) in pairs {
            hasher.update(b"\x01");
            hasher.update(name.as_bytes());
            hasher.update(data.as_bytes());
        }
    }
    crate::hashing::to_hex(hasher.finalize().as_slice())
}

pub fn add_inline_files(files: &mut HashMap<String, Vec<u8>>, inline_files: &Option<Vec<InlineFile>>) {
    for inline_file in inline_files.iter().flatten() {
        if let Ok(decoded) = BASE64.decode(&inline_file.data) {
            files.insert(inline_file.name.clone(), decoded);
        }
    }
}

pub async fn cached_render(state: &AppState, cache_key: &str, label: &str) -> Option<Vec<u8>> {
    let (data, created_at) = sqlx::query_as::<_, (Vec<u8>, String)>(sql(
        "SELECT data, created_at FROM api_render_cache WHERE content_hash = $1 AND format = $2",
    ))
    .bind(cache_key)
    .bind(label)
    .fetch_optional(&state.db)
    .await
    .ok()??;

    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at).ok()?;
    let age = chrono::Utc::now().signed_duration_since(created_at.with_timezone(&chrono::Utc));
    (age.num_seconds() < CACHE_MAX_AGE_SECONDS).then_some(data)
}

pub async fn store_render(state: &AppState, cache_key: &str, label: &str, data: &[u8]) {
    let _ = sqlx::query(sql(
        "INSERT INTO api_render_cache (id, content_hash, format, data, created_at) VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (content_hash) DO UPDATE SET format = excluded.format, data = excluded.data, created_at = excluded.created_at",
    ))
    .bind(Uuid::new_v4().to_string())
    .bind(cache_key)
    .bind(label)
    .bind(data.to_vec())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&state.db)
    .await;
}

pub async fn prune_cache(state: &AppState) {
    let now = chrono::Utc::now();
    let last_prune = LAST_CACHE_PRUNE.load(Ordering::Relaxed);
    if now.timestamp() - last_prune < CACHE_PRUNE_INTERVAL_SECONDS
        || LAST_CACHE_PRUNE.compare_exchange(last_prune, now.timestamp(), Ordering::Relaxed, Ordering::Relaxed).is_err()
    {
        return;
    }
    let cutoff = (now - chrono::TimeDelta::seconds(CACHE_MAX_AGE_SECONDS)).to_rfc3339();
    let _ = sqlx::query(sql("DELETE FROM api_render_cache WHERE created_at < $1"))
        .bind(cutoff)
        .execute(&state.db)
        .await;
}

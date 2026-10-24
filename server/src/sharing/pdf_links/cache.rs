use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use sha2::{Digest, Sha256};

use crate::compile::compiler::ProjectInput;

const MAX_CACHED_PDFS: usize = 64;

struct CachedPdf {
    fingerprint: String,
    bytes: Vec<u8>,
    stored_at: Instant,
}

fn cache() -> &'static Mutex<HashMap<String, CachedPdf>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CachedPdf>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn hash_files(hasher: &mut Sha256, files: &HashMap<String, Vec<u8>>) {
    let mut names: Vec<&String> = files.keys().collect();
    names.sort();
    for name in names {
        hasher.update(name.as_bytes());
        hasher.update(b"\x00");
        hasher.update((files[name].len() as u64).to_le_bytes());
        hasher.update(&files[name]);
    }
}

pub fn fingerprint(input: &ProjectInput) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.entrypoint.as_bytes());
    hasher.update(b"\x00");
    hash_files(&mut hasher, &input.files);
    let mut packages: Vec<&String> = input.packages.keys().collect();
    packages.sort();
    for package in packages {
        hasher.update(b"\x01");
        hasher.update(package.as_bytes());
        hash_files(&mut hasher, &input.packages[package]);
    }
    crate::hashing::to_hex(hasher.finalize().as_slice())
}

pub fn latest(link_id: &str) -> Option<(String, Vec<u8>)> {
    let cache = cache().lock().ok()?;
    cache.get(link_id).map(|cached| (cached.fingerprint.clone(), cached.bytes.clone()))
}

pub fn remember(link_id: &str, fingerprint: &str, bytes: &[u8]) {
    let Ok(mut cache) = cache().lock() else {
        return;
    };
    if cache.len() >= MAX_CACHED_PDFS && !cache.contains_key(link_id) {
        let oldest = cache
            .iter()
            .min_by_key(|(_, cached)| cached.stored_at)
            .map(|(key, _)| key.clone());
        if let Some(oldest) = oldest {
            cache.remove(&oldest);
        }
    }
    cache.insert(
        link_id.to_string(),
        CachedPdf { fingerprint: fingerprint.to_string(), bytes: bytes.to_vec(), stored_at: Instant::now() },
    );
}

pub fn forget(link_id: &str) {
    if let Ok(mut cache) = cache().lock() {
        cache.remove(link_id);
    }
}

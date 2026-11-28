use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Read;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use typst::diag::{FileError, FileResult};
use typst::foundations::Bytes;

use crate::compile::compiler::env_usize;
use crate::remote_hosts::resolve_remote_host;

const FETCH_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_FILE_BYTES: u64 = 50 * 1024 * 1024;
const MAX_REDIRECTS: u32 = 5;
const KEEP_FILES_FOR: Duration = Duration::from_secs(10 * 60);
const KEEP_FAILURES_FOR: Duration = Duration::from_secs(60);
const MAX_URLS_PER_COMPILE: usize = 50;
const MAX_BYTES_PER_COMPILE: usize = 150 * 1024 * 1024;

enum Fetched {
    File(Bytes),
    Failed(String),
}

struct CachedFetch {
    fetched: Fetched,
    stored_at: Instant,
}

impl CachedFetch {
    fn is_fresh(&self) -> bool {
        let keep_for = match self.fetched {
            Fetched::File(_) => KEEP_FILES_FOR,
            Fetched::Failed(_) => KEEP_FAILURES_FOR,
        };
        self.stored_at.elapsed() < keep_for
    }

    fn size(&self) -> usize {
        match &self.fetched {
            Fetched::File(bytes) => bytes.len(),
            Fetched::Failed(message) => message.len(),
        }
    }
}

#[derive(Default)]
struct RemoteCache {
    entries: HashMap<String, CachedFetch>,
    order: VecDeque<String>,
    total_bytes: usize,
}

static CACHE: LazyLock<Mutex<RemoteCache>> = LazyLock::new(|| Mutex::new(RemoteCache::default()));

fn cache_limit() -> usize {
    env_usize("REMOTE_CACHE_MB", 256).saturating_mul(1024 * 1024)
}

impl RemoteCache {
    fn get(&self, url: &str) -> Option<FileResult<Bytes>> {
        let cached = self.entries.get(url).filter(|cached| cached.is_fresh())?;
        Some(match &cached.fetched {
            Fetched::File(bytes) => Ok(bytes.clone()),
            Fetched::Failed(message) => Err(FileError::Other(Some(message.clone().into()))),
        })
    }

    fn remove(&mut self, url: &str) {
        if let Some(removed) = self.entries.remove(url) {
            self.total_bytes = self.total_bytes.saturating_sub(removed.size());
            self.order.retain(|kept| kept != url);
        }
    }

    fn insert(&mut self, url: &str, fetched: Fetched) {
        self.remove(url);
        let entry = CachedFetch { fetched, stored_at: Instant::now() };
        let limit = cache_limit();
        if entry.size() > limit / 4 {
            return;
        }
        self.total_bytes += entry.size();
        self.entries.insert(url.to_string(), entry);
        self.order.push_back(url.to_string());
        while self.total_bytes > limit {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            if let Some(removed) = self.entries.remove(&oldest) {
                self.total_bytes = self.total_bytes.saturating_sub(removed.size());
            }
        }
    }
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(FETCH_TIMEOUT)
        .redirects(MAX_REDIRECTS)
        .resolver(resolve_remote_host)
        .build();
    let response = agent.get(url).call().map_err(|error| format!("failed to fetch {url}: {error}"))?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(MAX_FILE_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("failed to read {url}: {error}"))?;
    Ok(bytes)
}

fn fetch(url: &str) -> FileResult<Bytes> {
    if let Some(cached) = CACHE.lock().ok().and_then(|cache| cache.get(url)) {
        return cached;
    }
    let fetched = match download(url) {
        Ok(bytes) => Fetched::File(Bytes::new(bytes)),
        Err(message) => Fetched::Failed(message),
    };
    let result = match &fetched {
        Fetched::File(bytes) => Ok(bytes.clone()),
        Fetched::Failed(message) => Err(FileError::Other(Some(message.clone().into()))),
    };
    if let Ok(mut cache) = CACHE.lock() {
        cache.insert(url, fetched);
    }
    result
}

#[derive(Default)]
pub struct RemoteBudget {
    used: Mutex<(HashSet<String>, usize)>,
}

impl RemoteBudget {
    pub fn fetch(&self, url: &str) -> FileResult<Bytes> {
        {
            let used = self.used.lock().map_err(|_| FileError::Other(None))?;
            if !used.0.contains(url) && used.0.len() >= MAX_URLS_PER_COMPILE {
                return Err(FileError::Other(Some(format!("a document can load at most {MAX_URLS_PER_COMPILE} web addresses").into())));
            }
        }
        let bytes = fetch(url)?;
        let mut used = self.used.lock().map_err(|_| FileError::Other(None))?;
        if used.0.insert(url.to_string()) {
            used.1 += bytes.len();
        }
        if used.1 > MAX_BYTES_PER_COMPILE {
            return Err(FileError::Other(Some(format!("a document can load at most {} MB from web addresses", MAX_BYTES_PER_COMPILE / 1024 / 1024).into())));
        }
        Ok(bytes)
    }
}

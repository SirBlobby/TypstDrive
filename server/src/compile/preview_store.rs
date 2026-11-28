
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::compile::compiler::{env_usize, DocumentStats, ProjectInput};
use crate::compile::speaker_notes::Note;
use crate::compile::world::TodayAnswer;
use crate::hashing::to_hex;

const FORMAT: u32 = 1;
const PREVIEWS_PER_TARGET: usize = 2;
const DEFAULT_MAX_MB: usize = 2000;

pub struct StoredPreview {
    pub svgs: Vec<String>,
    pub stats: DocumentStats,
    pub notes: Option<Vec<Option<Note>>>,
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    format: u32,
    build: String,
    key: String,
    target: String,
    stats: DocumentStats,
    #[serde(default)]
    notes: Option<Vec<Option<Note>>>,
    today: Vec<TodayAnswer>,
    pages: Vec<PageRef>,
}

#[derive(Serialize, Deserialize, Clone)]
struct PageRef {
    hash: String,
    len: u64,
}

struct Entry {
    target: String,
    pages: Vec<PageRef>,
    manifest_len: u64,
    last_used: SystemTime,
}

#[derive(Default)]
struct Index {
    entries: HashMap<String, Entry>,
    pages: HashMap<String, (u64, usize)>,
    total_bytes: u64,
}

struct Store {
    dir: PathBuf,
    max_bytes: u64,
    build: String,
    key_prefix: Vec<u8>,
    index: Mutex<Index>,
}

fn default_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("PREVIEW_CACHE_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir);
        }
    }
    if Path::new("/data").is_dir() {
        return PathBuf::from("/data/preview-cache");
    }
    PathBuf::from("preview-cache")
}

fn file_sha256(path: &Path) -> std::io::Result<String> {
    Ok(to_hex(Sha256::digest(fs::read(path)?).as_slice()))
}

fn store() -> Option<&'static Store> {
    static STORE: OnceLock<Option<Store>> = OnceLock::new();
    STORE.get_or_init(open_store).as_ref()
}

pub fn init() {
    store();
}

pub fn is_enabled() -> bool {
    store().is_some()
}

fn open_store() -> Option<Store> {
    let max_mb = env_usize("PREVIEW_CACHE_MB", DEFAULT_MAX_MB);
    if max_mb == 0 {
        tracing::info!("Preview store: turned off (PREVIEW_CACHE_MB=0)");
        return None;
    }
    let dir = default_dir();
    if let Err(error) = fs::create_dir_all(dir.join("entries")).and_then(|_| fs::create_dir_all(dir.join("pages"))) {
        tracing::warn!("Preview store: cannot use {}: {error}; previews are not kept", dir.display());
        return None;
    }
    let build = match std::env::current_exe().and_then(|exe| file_sha256(&exe)) {
        Ok(hash) => hash,
        Err(error) => {
            tracing::warn!("Preview store: cannot identify the server binary ({error}); previews are not kept");
            return None;
        }
    };

    let mut key_prefix = Vec::new();
    key_prefix.extend_from_slice(b"typstdrive-preview\0");
    key_prefix.extend_from_slice(&FORMAT.to_le_bytes());
    key_prefix.extend_from_slice(build.as_bytes());
    let fonts = typst::utils::hash128(&**crate::compile::fonts::shared_book());
    key_prefix.extend_from_slice(&fonts.to_le_bytes());
    key_prefix.extend_from_slice(b"svg-options:default\0");

    let store = Store {
        dir,
        max_bytes: (max_mb as u64).saturating_mul(1_000_000),
        build,
        key_prefix,
        index: Mutex::new(Index::default()),
    };
    store.load_index();
    Some(store)
}

fn hash_len_prefixed(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn hash_files(hasher: &mut Sha256, files: &HashMap<String, Vec<u8>>) {
    let mut names: Vec<&String> = files.keys().collect();
    names.sort();
    hasher.update((names.len() as u64).to_le_bytes());
    for name in names {
        hash_len_prefixed(hasher, name.as_bytes());
        hash_len_prefixed(hasher, &files[name]);
    }
}

pub fn key(input: &ProjectInput) -> Option<String> {
    let store = store()?;
    let mut hasher = Sha256::new();
    hash_len_prefixed(&mut hasher, &store.key_prefix);
    hash_len_prefixed(&mut hasher, input.entrypoint.as_bytes());
    hash_files(&mut hasher, &input.files);
    let mut packages: Vec<&String> = input.packages.keys().collect();
    packages.sort();
    hasher.update((packages.len() as u64).to_le_bytes());
    for package in packages {
        hash_len_prefixed(&mut hasher, package.as_bytes());
        hash_files(&mut hasher, &input.packages[package]);
    }
    Some(to_hex(hasher.finalize().as_slice()))
}

pub fn load(key: &str) -> Option<StoredPreview> {
    let store = store()?;
    let pages = {
        let index = store.index.lock().ok()?;
        index.entries.get(key)?.pages.clone()
    };
    match store.read(key, &pages) {
        Some(preview) => {
            store.touch(key);
            Some(preview)
        }
        None => {
            if let Ok(mut index) = store.index.lock() {
                store.remove_entry(&mut index, key);
            }
            None
        }
    }
}

pub fn save(key: &str, target: &str, svgs: &[String], stats: &DocumentStats, notes: &Option<Vec<Option<Note>>>, today: Vec<TodayAnswer>) {
    let Some(store) = store() else {
        return;
    };
    if let Err(error) = store.save(key, target, svgs, stats, notes, today) {
        tracing::warn!("Preview store: could not save a preview: {error}");
    }
}

pub fn stored_targets() -> Vec<String> {
    store().map(Store::targets).unwrap_or_default()
}

pub fn forget(target: &str) {
    if let Some(store) = store() {
        store.forget_target(target);
    }
}

fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let temporary = path.with_extension(format!("tmp{}", std::process::id()));
    let result = (|| {
        let mut file = fs::File::create(&temporary)?;
        file.write_all(bytes)?;
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

impl Store {
    fn entry_path(&self, key: &str) -> PathBuf {
        self.dir.join("entries").join(format!("{key}.json"))
    }

    fn page_path(&self, hash: &str) -> PathBuf {
        self.dir.join("pages").join(&hash[..2]).join(format!("{hash}.svg"))
    }

    fn read(&self, key: &str, pages: &[PageRef]) -> Option<StoredPreview> {
        let manifest: Manifest = serde_json::from_slice(&fs::read(self.entry_path(key)).ok()?).ok()?;
        if manifest.format != FORMAT || manifest.build != self.build || manifest.key != key {
            return None;
        }
        if !manifest.today.iter().all(TodayAnswer::still_holds) {
            return None;
        }
        let mut svgs = Vec::with_capacity(pages.len());
        for page in pages {
            let bytes = fs::read(self.page_path(&page.hash)).ok()?;
            if bytes.len() as u64 != page.len || to_hex(Sha256::digest(&bytes).as_slice()) != page.hash {
                return None;
            }
            svgs.push(String::from_utf8(bytes).ok()?);
        }
        Some(StoredPreview { svgs, stats: manifest.stats, notes: manifest.notes })
    }

    fn touch(&self, key: &str) {
        let now = SystemTime::now();
        if let Ok(mut index) = self.index.lock() {
            if let Some(entry) = index.entries.get_mut(key) {
                entry.last_used = now;
            }
        }
        if let Ok(file) = fs::File::options().append(true).open(self.entry_path(key)) {
            let _ = file.set_modified(now);
        }
    }

    fn save(&self, key: &str, target: &str, svgs: &[String], stats: &DocumentStats, notes: &Option<Vec<Option<Note>>>, today: Vec<TodayAnswer>) -> std::io::Result<()> {
        let pages: Vec<PageRef> = svgs
            .iter()
            .map(|svg| PageRef { hash: to_hex(Sha256::digest(svg.as_bytes()).as_slice()), len: svg.len() as u64 })
            .collect();
        let manifest = Manifest {
            format: FORMAT,
            build: self.build.clone(),
            key: key.to_string(),
            target: target.to_string(),
            stats: stats.clone(),
            notes: notes.clone(),
            today,
            pages: pages.clone(),
        };
        let manifest_bytes = serde_json::to_vec(&manifest).map_err(std::io::Error::other)?;

        let missing_pages: Vec<usize> = {
            let index = self.lock_index()?;
            if index.entries.get(key).is_some_and(|entry| entry.target == target) {
                drop(index);
                self.touch(key);
                return Ok(());
            }
            (0..pages.len()).filter(|&position| !index.pages.contains_key(&pages[position].hash)).collect()
        };
        for &position in &missing_pages {
            self.write_page(&pages[position].hash, &svgs[position])?;
        }

        let mut index = self.lock_index()?;
        for (position, (page, svg)) in pages.iter().zip(svgs).enumerate() {
            if !index.pages.contains_key(&page.hash) && !missing_pages.contains(&position) {
                self.write_page(&page.hash, svg)?;
            }
        }
        write_atomically(&self.entry_path(key), &manifest_bytes)?;

        let replaced = index.entries.remove(key);
        let entry = Entry { target: target.to_string(), pages, manifest_len: manifest_bytes.len() as u64, last_used: SystemTime::now() };
        Self::add_entry(&mut index, key.to_string(), entry);
        if let Some(replaced) = replaced {
            self.release(&mut index, &replaced);
        }

        let mut same_target: Vec<(String, SystemTime)> = index
            .entries
            .iter()
            .filter(|(other, entry)| entry.target == target && other.as_str() != key)
            .map(|(other, entry)| (other.clone(), entry.last_used))
            .collect();
        same_target.sort_by_key(|(_, used)| std::cmp::Reverse(*used));
        for (other, _) in same_target.into_iter().skip(PREVIEWS_PER_TARGET - 1) {
            self.remove_entry(&mut index, &other);
        }
        self.enforce_cap(&mut index, Some(key));
        Ok(())
    }

    fn lock_index(&self) -> std::io::Result<std::sync::MutexGuard<'_, Index>> {
        self.index.lock().map_err(|_| std::io::Error::other("preview index poisoned"))
    }

    fn write_page(&self, hash: &str, svg: &str) -> std::io::Result<()> {
        let path = self.page_path(hash);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        write_atomically(&path, svg.as_bytes())
    }

    fn targets(&self) -> Vec<String> {
        let Ok(index) = self.index.lock() else {
            return Vec::new();
        };
        let unique: HashSet<&String> = index.entries.values().map(|entry| &entry.target).collect();
        unique.into_iter().cloned().collect()
    }

    fn forget_target(&self, target: &str) {
        let Ok(mut index) = self.index.lock() else {
            return;
        };
        let keys: Vec<String> = index.entries.iter().filter(|(_, entry)| entry.target == target).map(|(key, _)| key.clone()).collect();
        for key in keys {
            self.remove_entry(&mut index, &key);
        }
    }

    fn add_entry(index: &mut Index, key: String, entry: Entry) {
        index.total_bytes += entry.manifest_len;
        for page in &entry.pages {
            let slot = index.pages.entry(page.hash.clone()).or_insert((page.len, 0));
            if slot.1 == 0 {
                index.total_bytes += page.len;
            }
            slot.1 += 1;
        }
        index.entries.insert(key, entry);
    }

    fn remove_entry(&self, index: &mut Index, key: &str) {
        let Some(entry) = index.entries.remove(key) else {
            return;
        };
        let _ = fs::remove_file(self.entry_path(key));
        self.release(index, &entry);
    }

    fn release(&self, index: &mut Index, entry: &Entry) {
        index.total_bytes = index.total_bytes.saturating_sub(entry.manifest_len);
        for page in &entry.pages {
            let Some(slot) = index.pages.get_mut(&page.hash) else {
                continue;
            };
            slot.1 = slot.1.saturating_sub(1);
            if slot.1 == 0 {
                index.total_bytes = index.total_bytes.saturating_sub(slot.0);
                index.pages.remove(&page.hash);
                let _ = fs::remove_file(self.page_path(&page.hash));
            }
        }
    }

    fn enforce_cap(&self, index: &mut Index, keep: Option<&str>) {
        if index.total_bytes <= self.max_bytes {
            return;
        }
        let mut by_age: Vec<(String, SystemTime)> = index.entries.iter().map(|(key, entry)| (key.clone(), entry.last_used)).collect();
        by_age.sort_by_key(|(_, used)| *used);
        for (key, _) in by_age {
            if index.total_bytes <= self.max_bytes {
                break;
            }
            if Some(key.as_str()) != keep {
                self.remove_entry(index, &key);
            }
        }
        if index.total_bytes > self.max_bytes {
            if let Some(keep) = keep {
                self.remove_entry(index, keep);
            }
        }
    }

    fn load_index(&self) {
        let mut index = Index::default();
        let mut dropped = 0;
        if let Ok(listing) = fs::read_dir(self.dir.join("entries")) {
            for item in listing.flatten() {
                let path = item.path();
                let name = item.file_name().to_string_lossy().to_string();
                let Some(key) = name.strip_suffix(".json").map(str::to_string) else {
                    let _ = fs::remove_file(&path);
                    continue;
                };
                let manifest = fs::read(&path).ok().and_then(|bytes| serde_json::from_slice::<Manifest>(&bytes).ok().map(|m| (m, bytes.len())));
                match manifest {
                    Some((manifest, len)) if manifest.format == FORMAT && manifest.build == self.build && manifest.key == key => {
                        let last_used = item.metadata().and_then(|meta| meta.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
                        let entry = Entry { target: manifest.target, pages: manifest.pages, manifest_len: len as u64, last_used };
                        Self::add_entry(&mut index, key, entry);
                    }
                    _ => {
                        let _ = fs::remove_file(&path);
                        dropped += 1;
                    }
                }
            }
        }
        let mut missing = Vec::new();
        if let Ok(shards) = fs::read_dir(self.dir.join("pages")) {
            for shard in shards.flatten() {
                let Ok(listing) = fs::read_dir(shard.path()) else {
                    continue;
                };
                for item in listing.flatten() {
                    let name = item.file_name().to_string_lossy().to_string();
                    let known = name.strip_suffix(".svg").and_then(|hash| index.pages.get(hash));
                    let complete = known.is_some_and(|(len, _)| item.metadata().is_ok_and(|meta| meta.len() == *len));
                    if !complete {
                        let _ = fs::remove_file(item.path());
                    }
                }
            }
        }
        for (key, entry) in &index.entries {
            if entry.pages.iter().any(|page| !self.page_path(&page.hash).is_file()) {
                missing.push(key.clone());
            }
        }
        for key in missing {
            self.remove_entry(&mut index, &key);
            dropped += 1;
        }
        self.enforce_cap(&mut index, None);
        tracing::info!(
            "Preview store: {} previews ({} MB of {} MB) in {}{}",
            index.entries.len(),
            index.total_bytes / 1_000_000,
            self.max_bytes / 1_000_000,
            self.dir.display(),
            if dropped > 0 { format!(", {dropped} stale ones dropped") } else { String::new() }
        );
        if let Ok(mut current) = self.index.lock() {
            *current = index;
        }
    }
}

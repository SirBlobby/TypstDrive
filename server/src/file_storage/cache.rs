use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

struct CacheEntries {
    data_by_key: HashMap<String, Vec<u8>>,
    insertion_order: VecDeque<String>,
    total_bytes: usize,
}

pub struct ObjectCache {
    limit_bytes: usize,
    entries: Mutex<CacheEntries>,
}

impl ObjectCache {
    pub fn new(limit_bytes: usize) -> Self {
        Self {
            limit_bytes,
            entries: Mutex::new(CacheEntries {
                data_by_key: HashMap::new(),
                insertion_order: VecDeque::new(),
                total_bytes: 0,
            }),
        }
    }

    pub fn limit_bytes(&self) -> usize {
        self.limit_bytes
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let entries = self.entries.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        entries.data_by_key.get(key).cloned()
    }

    pub fn insert(&self, key: &str, data: Vec<u8>) {
        if data.len() > self.limit_bytes / 4 {
            return;
        }
        let mut entries = self.entries.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if entries.data_by_key.contains_key(key) {
            return;
        }
        entries.total_bytes += data.len();
        entries.insertion_order.push_back(key.to_string());
        entries.data_by_key.insert(key.to_string(), data);

        while entries.total_bytes > self.limit_bytes {
            let Some(oldest_key) = entries.insertion_order.pop_front() else {
                break;
            };
            if let Some(removed) = entries.data_by_key.remove(&oldest_key) {
                entries.total_bytes -= removed.len();
            }
        }
    }
}

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use axum::http::StatusCode;
use tokio::sync::OwnedMutexGuard;

use super::ApiError;

type ProjectLocks = HashMap<String, Arc<tokio::sync::Mutex<()>>>;

static LOCKS: LazyLock<Mutex<ProjectLocks>> = LazyLock::new(Default::default);

pub fn lock_project(project_id: &str) -> Result<OwnedMutexGuard<()>, ApiError> {
    let lock = {
        let mut locks = LOCKS.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        locks.retain(|_, lock| Arc::strong_count(lock) > 1);
        locks.entry(project_id.to_string()).or_default().clone()
    };
    lock.try_lock_owned()
        .map_err(|_| (StatusCode::CONFLICT, "A Git sync is already running for this project. Try again in a moment.".to_string()))
}

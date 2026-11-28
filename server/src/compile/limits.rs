use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::Instant;

use crate::compile::compiler::env_usize;

const QUEUE_WAIT: Duration = Duration::from_secs(30);
const DEFAULT_PER_OWNER: usize = 3;
const DEFAULT_TIMEOUT_SECONDS: usize = 120;

tokio::task_local! {
    static COMPILE_OWNER: String;
}

pub async fn for_owner<F: Future>(owner: String, future: F) -> F::Output {
    COMPILE_OWNER.scope(owner, future).await
}

fn current_owner() -> Option<String> {
    COMPILE_OWNER.try_with(Clone::clone).ok()
}

fn default_running() -> usize {
    std::thread::available_parallelism().map_or(2, |count| count.get())
}

fn max_running() -> usize {
    env_usize("TYPST_MAX_COMPILES", default_running()).max(1)
}

fn max_per_owner() -> usize {
    env_usize("TYPST_MAX_COMPILES_PER_USER", DEFAULT_PER_OWNER).max(1)
}

pub fn time_limit() -> Duration {
    Duration::from_secs(env_usize("TYPST_COMPILE_TIMEOUT_SECONDS", DEFAULT_TIMEOUT_SECONDS).max(1) as u64)
}

static SLOTS: LazyLock<Arc<Semaphore>> = LazyLock::new(|| Arc::new(Semaphore::new(max_running())));
static OWNER_SLOTS: LazyLock<Mutex<HashMap<String, Arc<Semaphore>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn owner_semaphore(owner: &str) -> Arc<Semaphore> {
    let mut owners = OWNER_SLOTS.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let limit = max_per_owner();
    owners.retain(|_, semaphore| Arc::strong_count(semaphore) > 1 || semaphore.available_permits() < limit);
    owners.entry(owner.to_string()).or_insert_with(|| Arc::new(Semaphore::new(limit))).clone()
}

pub struct CompilePermit {
    _owner_slot: Option<OwnedSemaphorePermit>,
    _slot: OwnedSemaphorePermit,
}

pub struct Busy;

pub async fn acquire() -> Result<CompilePermit, Busy> {
    let deadline = Instant::now() + QUEUE_WAIT;
    let owner_slot = match current_owner() {
        Some(owner) => {
            let semaphore = owner_semaphore(&owner);
            match tokio::time::timeout_at(deadline, semaphore.acquire_owned()).await {
                Ok(Ok(permit)) => Some(permit),
                _ => {
                    tracing::warn!("Compile refused: {owner} already has {} compiles running", max_per_owner());
                    return Err(Busy);
                }
            }
        }
        None => None,
    };
    match tokio::time::timeout_at(deadline, SLOTS.clone().acquire_owned()).await {
        Ok(Ok(slot)) => Ok(CompilePermit { _owner_slot: owner_slot, _slot: slot }),
        _ => {
            tracing::warn!("Compile refused: all {} compile slots are in use", max_running());
            Err(Busy)
        }
    }
}

pub fn try_acquire_background() -> Option<CompilePermit> {
    let slot = SLOTS.clone().try_acquire_owned().ok()?;
    Some(CompilePermit { _owner_slot: None, _slot: slot })
}

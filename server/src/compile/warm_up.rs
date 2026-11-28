
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use crate::compile::compiler::{cache_max_idle, cache_memory_budget, env_usize, Preview, ProjectInput, TypstCompiler};
use crate::compile::preview_store;
use crate::db::sql;
use crate::AppState;

static COMPILED: LazyLock<Mutex<HashMap<String, Instant>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static RUNNING: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

pub fn note_compiled(target: &str) {
    if let Ok(mut compiled) = COMPILED.lock() {
        compiled.retain(|_, at| at.elapsed() < cache_max_idle());
        compiled.insert(target.to_string(), Instant::now());
    }
}

pub fn is_warm(target: &str) -> bool {
    COMPILED
        .lock()
        .map(|compiled| compiled.get(target).is_some_and(|at| at.elapsed() < cache_max_idle()))
        .unwrap_or(false)
}

struct Running(String);

impl Running {
    fn start(target: &str) -> Option<Self> {
        let mut running = RUNNING.lock().ok()?;
        running.insert(target.to_string()).then(|| Running(target.to_string()))
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        if let Ok(mut running) = RUNNING.lock() {
            running.remove(&self.0);
        }
    }
}

fn lower_thread_priority() {
    #[cfg(target_os = "linux")]
    unsafe {
        let thread = libc::syscall(libc::SYS_gettid) as libc::id_t;
        libc::setpriority(libc::PRIO_PROCESS, thread, 19);
    }
}

async fn compile_in_background(input: ProjectInput, key: Option<String>, target: String, low_priority: bool) -> bool {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let spawned = std::thread::Builder::new().name("typst-warm-up".into()).spawn(move || {
        if low_priority {
            lower_thread_priority();
        }
        let started = Instant::now();
        let result = TypstCompiler.compile_svg(input);
        let ok = result.is_ok();
        if let Ok(Preview { svgs, stats, notes, outside_dependencies: Some(today), .. }) = result {
            if let Some(key) = key {
                preview_store::save(&key, &target, &svgs, &stats, &notes, today);
            }
        }
        tracing::info!("Warm-up: compiled {target} in {} ms{}", started.elapsed().as_millis(), if ok { "" } else { " (with errors)" });
        let _ = sender.send(ok);
    });
    match spawned {
        Ok(_) => receiver.await.unwrap_or(false),
        Err(error) => {
            tracing::warn!("Warm-up: cannot start a compile thread: {error}");
            false
        }
    }
}

pub fn refresh(input: ProjectInput, key: String, target: String) {
    if is_warm(&target) {
        return;
    }
    let Some(running) = Running::start(&target) else {
        return;
    };
    tokio::spawn(async move {
        let _running = running;
        if compile_in_background(input, Some(key), target.clone(), true).await {
            note_compiled(&target);
        }
    });
}

struct WarmUpSettings {
    count: usize,
    max_age_days: usize,
    parallel: usize,
    delay: Duration,
}

impl WarmUpSettings {
    fn from_environment() -> Self {
        Self {
            count: env_usize("TYPST_WARMUP_COUNT", 4),
            max_age_days: env_usize("TYPST_WARMUP_MAX_AGE_DAYS", 7),
            parallel: env_usize("TYPST_WARMUP_PARALLEL", 1).max(1),
            delay: Duration::from_secs(env_usize("TYPST_WARMUP_DELAY_SECONDS", 10) as u64),
        }
    }
}

async fn recent_targets(state: &AppState, settings: &WarmUpSettings) -> Vec<(String, String, String)> {
    let since = (chrono::Utc::now() - chrono::Duration::days(settings.max_age_days as i64)).format("%Y-%m-%d %H:%M:%S").to_string();
    let limit = settings.count as i64;
    let mut targets = Vec::new();
    let queries = [
        ("project", "SELECT id, owner_id, updated_at FROM projects WHERE deleted_at IS NULL AND updated_at >= $1 ORDER BY updated_at DESC LIMIT $2"),
        ("document", "SELECT id, owner_id, updated_at FROM documents WHERE deleted_at IS NULL AND updated_at >= $1 ORDER BY updated_at DESC LIMIT $2"),
    ];
    for (kind, statement) in queries {
        let rows = sqlx::query_as::<_, (String, String, String)>(sql(statement))
            .bind(&since)
            .bind(limit)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();
        targets.extend(rows.into_iter().map(|(id, owner, updated)| (format!("{kind}:{id}"), owner, updated)));
    }
    targets.sort_by(|a, b| b.2.cmp(&a.2));
    targets.truncate(settings.count);
    targets
}

pub fn spawn_start_up_warm_up(state: AppState) {
    let settings = WarmUpSettings::from_environment();
    if settings.count == 0 {
        tracing::info!("Warm-up: turned off (TYPST_WARMUP_COUNT=0)");
        return;
    }
    tokio::spawn(async move {
        tokio::time::sleep(settings.delay).await;
        let started = Instant::now();
        let queue = Arc::new(Mutex::new(recent_targets(&state, &settings).await));
        let total = queue.lock().map(|queue| queue.len()).unwrap_or(0);
        tracing::info!("Warm-up: compiling {total} recently edited projects and documents, {} at a time", settings.parallel);
        let workers: Vec<_> = (0..settings.parallel.min(total.max(1)))
            .map(|_| {
                let state = state.clone();
                let queue = queue.clone();
                tokio::spawn(async move {
                    let mut warmed = 0;
                    loop {
                        let next = queue.lock().ok().and_then(|mut queue| (!queue.is_empty()).then(|| queue.remove(0)));
                        let Some((target, owner, _)) = next else {
                            break;
                        };
                        let budget = cache_memory_budget() / 10 * 7;
                        if crate::compile::compiler::resident_bytes().is_some_and(|rss| rss > budget) {
                            tracing::info!("Warm-up: stopped, the server already uses 70 % of its memory budget");
                            break;
                        }
                        if warm_up_one(&state, &target, owner).await {
                            warmed += 1;
                        }
                    }
                    warmed
                })
            })
            .collect();
        let mut warmed = 0;
        for worker in workers {
            warmed += worker.await.unwrap_or(0);
        }
        tracing::info!("Warm-up: {warmed} of {total} warmed in {} ms", started.elapsed().as_millis());
    });
}

async fn warm_up_one(state: &AppState, target: &str, owner: String) -> bool {
    if is_warm(target) {
        return false;
    }
    let Some(running) = Running::start(target) else {
        return false;
    };
    let (kind, id) = target.split_once(':').unwrap_or_default();
    let request = crate::compile::handlers::CompileRequest {
        text: None,
        document_id: (kind == "document").then(|| id.to_string()),
        project_id: (kind == "project").then(|| id.to_string()),
        files: None,
        known_pages: None,
    };
    let Some(source) = crate::compile::handlers::resolve_source(state, &request, &Some(owner)).await else {
        return false;
    };
    let input = source.input;
    if crate::compile::render::is_latex(&input) {
        return false;
    }
    let (input, key) = match tokio::task::spawn_blocking(move || {
        let key = preview_store::key(&input);
        (input, key)
    })
    .await
    {
        Ok(keyed) => keyed,
        Err(_) => return false,
    };
    let compiled = compile_in_background(input, key, target.to_string(), true).await;
    drop(running);
    if compiled {
        note_compiled(target);
    }
    compiled
}

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::cluster::ClusterEvent;
use crate::db::sql;
use crate::AppState;

type Settings = Arc<HashMap<String, String>>;

const CACHE_LIFETIME: Duration = Duration::from_secs(60);

static CACHED: RwLock<Option<(Settings, Instant)>> = RwLock::new(None);
static GENERATION: AtomicU64 = AtomicU64::new(0);

fn cached() -> Option<Settings> {
    let cache = CACHED.read().ok()?;
    let (settings, loaded_at) = cache.as_ref()?;
    (loaded_at.elapsed() < CACHE_LIFETIME).then(|| settings.clone())
}

pub fn forget_cached() {
    GENERATION.fetch_add(1, Ordering::SeqCst);
    if let Ok(mut cache) = CACHED.write() {
        *cache = None;
    }
}

async fn all_settings(state: &AppState) -> Settings {
    if let Some(settings) = cached() {
        return settings;
    }
    let generation = GENERATION.load(Ordering::SeqCst);
    let rows = sqlx::query_as::<_, (String, String)>(sql("SELECT setting_key, setting_value FROM site_settings"))
        .fetch_all(&state.db)
        .await;
    let Ok(rows) = rows else {
        return Settings::default();
    };
    let settings: Settings = Arc::new(rows.into_iter().collect());
    if let Ok(mut cache) = CACHED.write() {
        if GENERATION.load(Ordering::SeqCst) == generation {
            *cache = Some((settings.clone(), Instant::now()));
        }
    }
    settings
}

pub async fn load_settings(state: &AppState, prefix: &str) -> HashMap<String, String> {
    all_settings(state)
        .await
        .iter()
        .filter(|(key, _)| key.starts_with(prefix))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

pub async fn save_settings(state: &AppState, entries: &[(&str, String)]) -> Result<(), sqlx::Error> {
    let mut transaction = state.db.begin().await?;
    for (key, value) in entries {
        sqlx::query(
            sql("INSERT INTO site_settings (setting_key, setting_value) VALUES ($1, $2) \
             ON CONFLICT (setting_key) DO UPDATE SET setting_value = excluded.setting_value")
        )
        .bind(*key)
        .bind(value)
        .execute(&mut *transaction)
        .await?;
    }
    transaction.commit().await?;
    forget_cached();
    state.cluster.publish_event(&ClusterEvent::ReloadSettings);
    Ok(())
}

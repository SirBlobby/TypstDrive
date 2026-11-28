use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use crate::db::sql;
use crate::AppState;

const FLUSH_INTERVAL: Duration = Duration::from_secs(5);
const DETAIL_KEPT_HOURS: i64 = 25;

#[derive(Hash, PartialEq, Eq, Clone)]
struct UsageSlot {
    key_id: String,
    user_id: String,
    date: String,
    minute: String,
}

#[derive(Default)]
struct PendingUsage {
    counts: HashMap<UsageSlot, i64>,
    last_used: HashMap<String, String>,
}

static PENDING: LazyLock<Mutex<PendingUsage>> = LazyLock::new(|| Mutex::new(PendingUsage::default()));

pub fn note(key_id: &str, user_id: &str) {
    let now = chrono::Utc::now();
    let slot = UsageSlot {
        key_id: key_id.to_string(),
        user_id: user_id.to_string(),
        date: now.format("%Y-%m-%d").to_string(),
        minute: now.format("%Y-%m-%d %H:%M").to_string(),
    };
    let mut pending = PENDING.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    *pending.counts.entry(slot).or_insert(0) += 1;
    pending.last_used.insert(key_id.to_string(), now.to_rfc3339());
}

pub fn pending_today(user_id: &str) -> i64 {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let pending = PENDING.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    pending.counts.iter().filter(|(slot, _)| slot.user_id == user_id && slot.date == today).map(|(_, count)| count).sum()
}

fn take_pending() -> PendingUsage {
    let mut pending = PENDING.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    std::mem::take(&mut *pending)
}

async fn write(state: &AppState, usage: PendingUsage) {
    for (key_id, last_used) in usage.last_used {
        let _ = sqlx::query(sql("UPDATE api_keys SET last_used_at = $1 WHERE id = $2"))
            .bind(last_used)
            .bind(key_id)
            .execute(&state.db)
            .await;
    }
    let mut daily: HashMap<(String, String), i64> = HashMap::new();
    for (slot, count) in &usage.counts {
        *daily.entry((slot.key_id.clone(), slot.date.clone())).or_insert(0) += count;
        let _ = sqlx::query(sql(
            "INSERT INTO api_key_usage_detail (key_id, minute, count) VALUES ($1, $2, $3) \
             ON CONFLICT (key_id, minute) DO UPDATE SET count = api_key_usage_detail.count + excluded.count",
        ))
        .bind(&slot.key_id)
        .bind(&slot.minute)
        .bind(count)
        .execute(&state.db)
        .await;
    }
    for ((key_id, date), count) in daily {
        let _ = sqlx::query(sql(
            "INSERT INTO api_key_usage (key_id, date, count) VALUES ($1, $2, $3) \
             ON CONFLICT (key_id, date) DO UPDATE SET count = api_key_usage.count + excluded.count",
        ))
        .bind(key_id)
        .bind(date)
        .bind(count)
        .execute(&state.db)
        .await;
    }
}

async fn prune_detail(state: &AppState) {
    let cutoff = (chrono::Utc::now() - chrono::TimeDelta::hours(DETAIL_KEPT_HOURS)).format("%Y-%m-%d %H:%M").to_string();
    let _ = sqlx::query(sql("DELETE FROM api_key_usage_detail WHERE minute < $1")).bind(cutoff).execute(&state.db).await;
}

pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        let mut ticks: u64 = 0;
        loop {
            tokio::time::sleep(FLUSH_INTERVAL).await;
            let usage = take_pending();
            if !usage.counts.is_empty() {
                write(&state, usage).await;
            }
            ticks += 1;
            if ticks.is_multiple_of(12) {
                prune_detail(&state).await;
            }
        }
    });
}

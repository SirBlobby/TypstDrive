use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::http::StatusCode;

const ADDRESS_FAILURE_MULTIPLIER: u32 = 4;
const MAX_TRACKED_KEYS: usize = 10_000;

#[derive(Clone, Copy)]
pub struct LockoutRules {
    pub max_failures: u32,
    pub lockout: Duration,
}

impl LockoutRules {
    pub fn is_enabled(&self) -> bool {
        self.max_failures > 0
    }
}

struct FailureRecord {
    count: u32,
    window_started: Instant,
    locked_until: Option<Instant>,
}

#[derive(Default)]
pub struct LoginGuard {
    failures: Mutex<HashMap<String, FailureRecord>>,
}

fn account_key(scope: &str, account: &str) -> String {
    format!("{scope}:account:{}", account.trim().to_lowercase())
}

fn address_key(scope: &str, ip_address: &str) -> String {
    format!("{scope}:address:{ip_address}")
}

fn locked_error(remaining: Duration) -> (StatusCode, String) {
    let minutes = remaining.as_secs().div_ceil(60).max(1);
    (
        StatusCode::TOO_MANY_REQUESTS,
        format!("Too many failed attempts. Try again in {minutes} minute(s)."),
    )
}

impl LoginGuard {
    fn remaining_lock(&self, key: &str) -> Option<Duration> {
        let failures = self.failures.lock().ok()?;
        let locked_until = failures.get(key)?.locked_until?;
        locked_until.checked_duration_since(Instant::now())
    }

    pub fn ensure_allowed(&self, scope: &str, account: &str, ip_address: &str) -> Result<(), (StatusCode, String)> {
        for key in [account_key(scope, account), address_key(scope, ip_address)] {
            if let Some(remaining) = self.remaining_lock(&key) {
                return Err(locked_error(remaining));
            }
        }
        Ok(())
    }

    fn count_failure(&self, key: String, max_failures: u32, lockout: Duration) {
        let Ok(mut failures) = self.failures.lock() else {
            return;
        };
        let now = Instant::now();
        if failures.len() > MAX_TRACKED_KEYS {
            failures.retain(|_, record| now.duration_since(record.window_started) < lockout);
        }

        let record = failures.entry(key).or_insert(FailureRecord { count: 0, window_started: now, locked_until: None });
        let lock_expired = record.locked_until.is_some_and(|until| until <= now);
        if lock_expired || now.duration_since(record.window_started) > lockout {
            *record = FailureRecord { count: 0, window_started: now, locked_until: None };
        }

        record.count += 1;
        if record.count >= max_failures {
            record.locked_until = Some(now + lockout);
        }
    }

    pub fn record_failure(&self, scope: &str, account: &str, ip_address: &str, rules: LockoutRules) {
        if !rules.is_enabled() {
            return;
        }
        self.count_failure(account_key(scope, account), rules.max_failures, rules.lockout);
        self.count_failure(
            address_key(scope, ip_address),
            rules.max_failures.saturating_mul(ADDRESS_FAILURE_MULTIPLIER),
            rules.lockout,
        );
    }

    pub fn record_success(&self, scope: &str, account: &str) {
        if let Ok(mut failures) = self.failures.lock() {
            failures.remove(&account_key(scope, account));
        }
    }
}

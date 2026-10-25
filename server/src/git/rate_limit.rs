use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use axum::http::StatusCode;

use super::ApiError;

const REQUESTS_PER_MINUTE: u32 = 30;
const WINDOW: Duration = Duration::from_secs(60);

static WINDOWS: LazyLock<Mutex<HashMap<String, (u32, Instant)>>> = LazyLock::new(Default::default);

pub fn check_rate(user_id: &str) -> Result<(), ApiError> {
    let mut windows = WINDOWS.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let now = Instant::now();
    windows.retain(|_, (_, started)| now.duration_since(*started) < WINDOW);
    let window = windows.entry(user_id.to_string()).or_insert((0, now));
    if window.0 >= REQUESTS_PER_MINUTE {
        return Err((StatusCode::TOO_MANY_REQUESTS, "Too many Git requests. Wait a minute and try again.".to_string()));
    }
    window.0 += 1;
    Ok(())
}

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use crate::compile::compiler::env_usize;

#[derive(Default)]
struct SessionCounts {
    total: usize,
    per_user: HashMap<String, usize>,
}

static SESSIONS: LazyLock<Mutex<SessionCounts>> = LazyLock::new(|| Mutex::new(SessionCounts::default()));

fn max_sessions() -> usize {
    env_usize("LSP_MAX_SESSIONS", 64)
}

fn max_sessions_per_user() -> usize {
    env_usize("LSP_MAX_SESSIONS_PER_USER", 4)
}

pub struct LspSlot {
    user_id: Option<String>,
}

impl LspSlot {
    pub fn acquire(user_id: &Option<String>) -> Option<Self> {
        let mut sessions = SESSIONS.lock().ok()?;
        if sessions.total >= max_sessions() {
            return None;
        }
        if let Some(user) = user_id {
            let used = sessions.per_user.get(user).copied().unwrap_or(0);
            if used >= max_sessions_per_user() {
                return None;
            }
            sessions.per_user.insert(user.clone(), used + 1);
        }
        sessions.total += 1;
        Some(Self { user_id: user_id.clone() })
    }
}

impl Drop for LspSlot {
    fn drop(&mut self) {
        let Ok(mut sessions) = SESSIONS.lock() else {
            return;
        };
        sessions.total = sessions.total.saturating_sub(1);
        if let Some(user) = &self.user_id {
            if let Some(used) = sessions.per_user.get_mut(user) {
                *used = used.saturating_sub(1);
                if *used == 0 {
                    sessions.per_user.remove(user);
                }
            }
        }
    }
}

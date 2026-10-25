use std::collections::HashMap;

use super::palette::Theme;
use super::{policy, store};
use crate::cluster::ClusterEvent;
use crate::AppState;

#[derive(Default)]
pub struct ThemeCache {
    pub site: Vec<Theme>,
    pub personal: HashMap<String, Vec<Theme>>,
    pub personal_enabled: bool,
}

impl ThemeCache {
    pub fn personal_for(&self, user_id: &str) -> &[Theme] {
        self.personal.get(user_id).map(Vec::as_slice).unwrap_or_default()
    }
}

pub async fn load(state: &AppState) {
    reload_site(state).await;
    match store::all_personal_themes(&state.db).await {
        Ok(rows) => {
            let mut personal: HashMap<String, Vec<Theme>> = HashMap::new();
            for (owner_id, theme) in rows {
                personal.entry(owner_id).or_default().push(theme);
            }
            state.themes.write().await.personal = personal;
        }
        Err(error) => tracing::warn!("Could not load personal themes: {error}"),
    }
}

async fn reload_site(state: &AppState) {
    let personal_enabled = policy::personal_enabled(state).await;
    let site = store::site_themes(&state.db).await;
    let mut cache = state.themes.write().await;
    cache.personal_enabled = personal_enabled;
    match site {
        Ok(site) => cache.site = site,
        Err(error) => tracing::warn!("Could not load site themes: {error}"),
    }
}

async fn reload_personal(state: &AppState, user_id: &str) {
    match store::personal_themes(&state.db, user_id).await {
        Ok(themes) => {
            let mut cache = state.themes.write().await;
            if themes.is_empty() {
                cache.personal.remove(user_id);
            } else {
                cache.personal.insert(user_id.to_string(), themes);
            }
        }
        Err(error) => tracing::warn!("Could not load themes for {user_id}: {error}"),
    }
}

pub async fn reload(state: &AppState, owner_id: Option<&str>) {
    match owner_id {
        Some(user_id) => reload_personal(state, user_id).await,
        None => reload_site(state).await,
    }
}

pub async fn changed(state: &AppState, owner_id: Option<&str>) {
    reload(state, owner_id).await;
    state.cluster.publish_event(&ClusterEvent::ReloadThemes { owner_id: owner_id.map(str::to_string) });
}

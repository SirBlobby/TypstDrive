use std::collections::BTreeMap;

use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;

use super::access::editable_project;
use super::links::{require_link, LinkView};
use super::local_files;
use super::locks::lock_project;
use super::policy::load_policy;
use super::remote;
use super::snapshot;
use super::sync_plan::{as_list, local_changes, remote_changes, Change};
use super::ApiError;
use crate::accounts::signed_in::SignedInUser;
use crate::AppState;

#[derive(Serialize)]
pub struct ChangesView {
    pub link: LinkView,
    pub head: String,
    pub behind: bool,
    pub local: Vec<Change>,
    pub remote: Vec<Change>,
    pub conflicts: Vec<String>,
}

pub async fn list_changes(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<Json<ChangesView>, ApiError> {
    let access = editable_project(&state, &project_id, &user_id).await?;
    let link = require_link(&state, &project_id).await?;
    load_policy(&state).await.check_provider(link.provider)?;
    let _guard = lock_project(&project_id)?;

    let repo = link.remote();
    let rules = link.rules();
    let head = remote::head(&repo).await?;
    let behind = link.base_commit.as_deref() != Some(head.as_str());
    let saved = snapshot::load(&state, &project_id).await?;
    let local_changed = local_changes(&local_files::load(&state, &project_id, &rules).await?, &saved, &rules);
    let remote_changed = if behind {
        remote_changes(&remote::list_files(&repo, &head).await?, &saved, &rules)
    } else {
        BTreeMap::new()
    };
    let conflicts = local_changed.keys().filter(|path| remote_changed.contains_key(*path)).cloned().collect();

    Ok(Json(ChangesView {
        link: link.view(access.is_owner),
        head,
        behind,
        local: as_list(&local_changed),
        remote: as_list(&remote_changed),
        conflicts,
    }))
}

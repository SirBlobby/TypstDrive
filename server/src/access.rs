use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::http::StatusCode;
use tokio::sync::oneshot;

use crate::models::{Document, Project};
use crate::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessRole {
    Owner,
    Editor,
    Viewer,
}

impl AccessRole {
    fn from_share_role(role: &str) -> Option<Self> {
        match role {
            "editor" => Some(Self::Editor),
            "viewer" => Some(Self::Viewer),
            _ => None,
        }
    }

    pub fn can_edit(self) -> bool {
        matches!(self, Self::Owner | Self::Editor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Resource {
    Document(String),
    Project(String),
}

pub struct LiveSession {
    id: u64,
    user_id: Option<String>,
    device_token_id: Option<String>,
    can_edit: bool,
    revoke: Option<oneshot::Sender<()>>,
}

impl LiveSession {
    fn revoke(&mut self) {
        if let Some(sender) = self.revoke.take() {
            let _ = sender.send(());
        }
    }
}

pub type LiveSessions = HashMap<Resource, Vec<LiveSession>>;

pub struct SessionHandle {
    pub resource: Resource,
    pub session_id: u64,
    pub revoked: oneshot::Receiver<()>,
}

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

pub fn forbidden() -> (StatusCode, String) {
    (StatusCode::FORBIDDEN, "You do not have access to this item".to_string())
}

pub async fn document_role(
    state: &AppState,
    document_id: &str,
    user_id: &Option<String>,
) -> Option<(Document, AccessRole)> {
    let document = sqlx::query_as::<_, Document>(
        "SELECT id, owner_id, folder_id, title, content, thumbnail_svg, public_role, created_at, updated_at FROM documents WHERE id = $1"
    )
    .bind(document_id)
    .fetch_optional(&state.db)
    .await
    .ok()??;

    if let Some(uid) = user_id {
        if &document.owner_id == uid {
            return Some((document, AccessRole::Owner));
        }
        let collaborator_role = sqlx::query_as::<_, (String,)>(
            "SELECT role FROM collaborators WHERE document_id = $1 AND user_id = $2"
        )
        .bind(document_id)
        .bind(uid)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .and_then(|(role,)| AccessRole::from_share_role(&role));
        if let Some(role) = collaborator_role {
            return Some((document, role));
        }
    }

    let public_role = document.public_role.as_deref().and_then(AccessRole::from_share_role);
    public_role.map(|role| (document, role))
}

pub async fn project_access(
    state: &AppState,
    project_id: &str,
    user_id: &Option<String>,
) -> Option<(Project, AccessRole)> {
    let (project, role) = crate::projects::project_role(state, project_id, user_id).await?;
    let role = if role == "owner" {
        AccessRole::Owner
    } else {
        AccessRole::from_share_role(&role)?
    };
    Some((project, role))
}

async fn current_role(state: &AppState, resource: &Resource, user_id: &Option<String>) -> Option<AccessRole> {
    match resource {
        Resource::Document(id) => document_role(state, id, user_id).await.map(|(_, role)| role),
        Resource::Project(id) => project_access(state, id, user_id).await.map(|(_, role)| role),
    }
}

pub async fn register_session(
    state: &AppState,
    resource: Resource,
    user_id: Option<String>,
    device_token_id: Option<String>,
    can_edit: bool,
) -> SessionHandle {
    let session_id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
    let (sender, receiver) = oneshot::channel();
    let mut sessions = state.live_sessions.lock().await;
    sessions.entry(resource.clone()).or_default().push(LiveSession {
        id: session_id,
        user_id,
        device_token_id,
        can_edit,
        revoke: Some(sender),
    });
    SessionHandle { resource, session_id, revoked: receiver }
}

pub async fn unregister_session(state: &AppState, resource: &Resource, session_id: u64) {
    let mut sessions = state.live_sessions.lock().await;
    if let Some(list) = sessions.get_mut(resource) {
        list.retain(|session| session.id != session_id);
        if list.is_empty() {
            sessions.remove(resource);
        }
    }
}

pub async fn revalidate_resource(state: &AppState, resource: Resource) {
    let checks: Vec<(u64, Option<String>, bool)> = {
        let sessions = state.live_sessions.lock().await;
        match sessions.get(&resource) {
            Some(list) => list.iter().map(|s| (s.id, s.user_id.clone(), s.can_edit)).collect(),
            None => return,
        }
    };

    let mut stale_session_ids = Vec::new();
    for (session_id, user_id, had_edit_access) in checks {
        let still_matches = match current_role(state, &resource, &user_id).await {
            Some(role) => role.can_edit() == had_edit_access,
            None => false,
        };
        if !still_matches {
            stale_session_ids.push(session_id);
        }
    }

    revoke_matching(state, |key, session| key == &resource && stale_session_ids.contains(&session.id)).await;
}

pub async fn revoke_user_sessions(state: &AppState, user_id: &str) {
    revoke_matching(state, |_, session| session.user_id.as_deref() == Some(user_id)).await;
}

pub async fn revoke_device_sessions(state: &AppState, device_token_id: &str) {
    revoke_matching(state, |_, session| session.device_token_id.as_deref() == Some(device_token_id)).await;
}

async fn revoke_matching(state: &AppState, should_revoke: impl Fn(&Resource, &LiveSession) -> bool) {
    let mut sessions = state.live_sessions.lock().await;
    for (resource, list) in sessions.iter_mut() {
        for session in list.iter_mut() {
            if should_revoke(resource, session) {
                session.revoke();
            }
        }
        list.retain(|session| session.revoke.is_some());
    }
    sessions.retain(|_, list| !list.is_empty());
}

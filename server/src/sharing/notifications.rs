use crate::db::{sql, sql_owned};
use crate::email::messages::{self, SharedItem};
use crate::sharing::members::ShareKind;
use crate::AppState;

fn name_column(kind: ShareKind) -> &'static str {
    match kind {
        ShareKind::Document => "title",
        ShareKind::Project | ShareKind::Folder => "name",
    }
}

fn item_path(kind: ShareKind, item_id: &str) -> String {
    match kind {
        ShareKind::Document => format!("/doc/{item_id}"),
        ShareKind::Project => format!("/project/{item_id}"),
        ShareKind::Folder => "/drive?view=shared".to_string(),
    }
}

async fn item_name(state: &AppState, kind: ShareKind, item_id: &str) -> Option<String> {
    let statement = format!("SELECT {} FROM {} WHERE id = $1", name_column(kind), kind.item_table());
    sqlx::query_as::<_, (String,)>(sql_owned(statement))
        .bind(item_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .map(|(name,)| name)
}

async fn account_name(state: &AppState, account_id: &str) -> String {
    sqlx::query_as::<_, (String,)>(sql("SELECT username FROM users WHERE id = $1"))
        .bind(account_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .map(|(name,)| name)
        .unwrap_or_else(|| "Someone".to_string())
}

async fn recipient(state: &AppState, user_id: &str) -> Option<(String, String)> {
    let user = crate::accounts::lookup::user_by_id(state, user_id).await.ok().flatten()?;
    (!user.email.trim().is_empty()).then_some((user.username, user.email))
}

pub fn item_shared(state: &AppState, kind: ShareKind, item_id: &str, inviter_id: &str, invited_user_id: &str, role: &str) {
    if !state.mailer.is_configured() {
        return;
    }
    let (state, item_id, inviter_id, invited_user_id, role) = (state.clone(), item_id.to_string(), inviter_id.to_string(), invited_user_id.to_string(), role.to_string());
    tokio::spawn(async move {
        let (Some((username, email)), Some(name)) = (recipient(&state, &invited_user_id).await, item_name(&state, kind, &item_id).await) else {
            return;
        };
        let inviter_name = account_name(&state, &inviter_id).await;
        let link = crate::site_url::link(&item_path(kind, &item_id));
        let item = SharedItem { kind: kind.label(), name: &name, link: link.as_deref() };
        let message = messages::item_shared(&crate::email::brand::load(&state).await, &username, &inviter_name, item, &role);
        if let Err(error) = state.mailer.send(&email, message).await {
            tracing::warn!("Sharing email to {email} failed: {error}");
        }
    });
}

pub fn ownership_offered(state: &AppState, kind: ShareKind, item_id: &str, owner_id: &str, to_user_id: &str) {
    if !state.mailer.is_configured() {
        return;
    }
    let (state, item_id, owner_id, to_user_id) = (state.clone(), item_id.to_string(), owner_id.to_string(), to_user_id.to_string());
    tokio::spawn(async move {
        let (Some((username, email)), Some(name)) = (recipient(&state, &to_user_id).await, item_name(&state, kind, &item_id).await) else {
            return;
        };
        let owner_name = account_name(&state, &owner_id).await;
        let link = crate::site_url::link("/drive?view=shared");
        let item = SharedItem { kind: kind.label(), name: &name, link: link.as_deref() };
        let message = messages::ownership_offered(&crate::email::brand::load(&state).await, &username, &owner_name, item);
        if let Err(error) = state.mailer.send(&email, message).await {
            tracing::warn!("Ownership email to {email} failed: {error}");
        }
    });
}

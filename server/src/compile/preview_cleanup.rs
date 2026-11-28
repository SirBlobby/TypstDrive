use crate::compile::preview_store;
use crate::db::sql;
use crate::AppState;

async fn target_exists(state: &AppState, target: &str) -> bool {
    let query = match target.split_once(':') {
        Some(("document", _)) => sql("SELECT 1 FROM documents WHERE id = $1"),
        Some(("project", _)) => sql("SELECT 1 FROM projects WHERE id = $1"),
        _ => return false,
    };
    let id = target.split_once(':').map(|(_, id)| id).unwrap_or_default();
    match sqlx::query_scalar::<_, i32>(query).bind(id).fetch_optional(&state.db).await {
        Ok(found) => found.is_some(),
        Err(_) => true,
    }
}

pub async fn forget_deleted(state: &AppState) {
    let targets = tokio::task::spawn_blocking(preview_store::stored_targets).await.unwrap_or_default();
    let mut forgotten = Vec::new();
    for target in targets {
        if !target_exists(state, &target).await {
            forgotten.push(target);
        }
    }
    if forgotten.is_empty() {
        return;
    }
    let count = forgotten.len();
    let _ = tokio::task::spawn_blocking(move || {
        for target in &forgotten {
            preview_store::forget(target);
        }
    })
    .await;
    tracing::info!("Preview store: dropped the previews of {count} deleted documents and projects");
}

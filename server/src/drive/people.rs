use std::collections::HashMap;

use serde::Serialize;

use crate::db::sql_owned;
use crate::AppState;

#[derive(Serialize, Clone)]
pub struct PersonSummary {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
}

pub async fn load_people(state: &AppState, ids: &[String]) -> HashMap<String, PersonSummary> {
    let mut unique_ids = ids.to_vec();
    unique_ids.sort();
    unique_ids.dedup();
    if unique_ids.is_empty() {
        return HashMap::new();
    }

    let placeholders = (1..=unique_ids.len()).map(|index| format!("${index}")).collect::<Vec<_>>().join(", ");
    let statement = format!(
        "SELECT u.id, u.username, a.updated_at FROM users u LEFT JOIN user_avatars a ON a.user_id = u.id WHERE u.id IN ({placeholders})"
    );
    let mut query = sqlx::query_as::<_, (String, String, Option<String>)>(sql_owned(statement));
    for id in &unique_ids {
        query = query.bind(id);
    }
    query
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(id, name, avatar_updated_at)| {
            let avatar_url = avatar_updated_at.map(|updated_at| format!("/api/users/{id}/avatar?v={updated_at}"));
            (id.clone(), PersonSummary { id, name, avatar_url })
        })
        .collect()
}

use serde::Serialize;

use crate::db::{sql, sql_owned};
use crate::sharing::members::ShareKind;
use crate::AppState;

#[derive(Serialize)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub is_team: bool,
}

#[derive(Serialize)]
pub struct AccessEntry {
    pub name: String,
    pub email: Option<String>,
    pub role: String,
    pub source: &'static str,
    pub via: Option<String>,
}

pub async fn person(state: &AppState, account_id: &str) -> Option<Person> {
    let (username, email, account_kind, team_name) = sqlx::query_as::<_, (String, Option<String>, String, Option<String>)>(sql(
        "SELECT u.username, u.email, u.account_kind, t.name FROM users u LEFT JOIN teams t ON t.id = u.id WHERE u.id = $1",
    ))
    .bind(account_id)
    .fetch_optional(&state.db)
    .await
    .ok()??;

    let is_team = account_kind == "team";
    Some(Person {
        id: account_id.to_string(),
        name: if is_team { team_name.unwrap_or(username) } else { username },
        email: if is_team { None } else { email },
        is_team,
    })
}

pub async fn folder_path(state: &AppState, folder_id: &str) -> Vec<String> {
    sqlx::query_as::<_, (String,)>(sql(
        "WITH RECURSIVE ancestors(id, parent_id, name, depth) AS ( \
         SELECT id, parent_id, name, 0 FROM folders WHERE id = $1 \
         UNION ALL SELECT f.id, f.parent_id, f.name, a.depth + 1 FROM folders f INNER JOIN ancestors a ON f.id = a.parent_id \
         ) SELECT name FROM ancestors ORDER BY depth DESC",
    ))
    .bind(folder_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(name,)| name)
    .collect()
}

async fn team_members(state: &AppState, team_id: &str) -> Vec<AccessEntry> {
    sqlx::query_as::<_, (String, Option<String>, String)>(sql(
        "SELECT u.username, u.email, tm.role FROM team_members tm INNER JOIN users u ON u.id = tm.user_id \
         WHERE tm.team_id = $1 ORDER BY u.username ASC",
    ))
    .bind(team_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(name, email, role)| AccessEntry { name, email, role, source: "team", via: None })
    .collect()
}

async fn direct_collaborators(state: &AppState, kind: ShareKind, item_id: &str) -> Vec<AccessEntry> {
    let statement = format!(
        "SELECT u.username, u.email, c.role FROM {} c INNER JOIN users u ON u.id = c.user_id \
         WHERE c.{} = $1 ORDER BY u.username ASC",
        kind.collaborator_table(),
        kind.item_column()
    );
    sqlx::query_as::<_, (String, Option<String>, String)>(sql_owned(statement))
        .bind(item_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(name, email, role)| AccessEntry { name, email, role, source: "shared", via: None })
        .collect()
}

async fn folder_collaborators(state: &AppState, folder_id: &str) -> Vec<AccessEntry> {
    sqlx::query_as::<_, (String, Option<String>, String, String)>(sql(
        "WITH RECURSIVE ancestors(id, parent_id) AS ( \
         SELECT id, parent_id FROM folders WHERE id = $1 \
         UNION ALL SELECT f.id, f.parent_id FROM folders f INNER JOIN ancestors a ON f.id = a.parent_id \
         ) SELECT u.username, u.email, c.role, fo.name FROM ancestors a \
         INNER JOIN folder_collaborators c ON c.folder_id = a.id \
         INNER JOIN users u ON u.id = c.user_id \
         INNER JOIN folders fo ON fo.id = a.id \
         ORDER BY u.username ASC",
    ))
    .bind(folder_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(name, email, role, folder_name)| AccessEntry { name, email, role, source: "folder", via: Some(folder_name) })
    .collect()
}

pub async fn who_has_access(
    state: &AppState,
    owner: &Person,
    share_kind: Option<ShareKind>,
    item_id: &str,
    folder_id: Option<&str>,
) -> Vec<AccessEntry> {
    let mut entries = vec![AccessEntry {
        name: owner.name.clone(),
        email: owner.email.clone(),
        role: "owner".to_string(),
        source: "owner",
        via: None,
    }];
    if owner.is_team {
        entries.extend(team_members(state, &owner.id).await);
    }
    if let Some(kind) = share_kind {
        entries.extend(direct_collaborators(state, kind, item_id).await);
    }
    if let Some(folder_id) = folder_id {
        entries.extend(folder_collaborators(state, folder_id).await);
    }
    entries
}

mod access_list;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Serialize;

use access_list::{folder_path, person, who_has_access, AccessEntry, Person};
use crate::db::sql;
use crate::sharing::access::{document_role, folder_role};
use crate::sharing::members::ShareKind;
use crate::AppState;

type ApiError = (StatusCode, String);

#[derive(Serialize)]
pub struct ItemInfo {
    pub kind: String,
    pub name: String,
    pub owner: Person,
    pub your_role: String,
    pub size_bytes: i64,
    pub location: Option<Vec<String>>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub last_edited_by: Option<Person>,
    pub mime_type: Option<String>,
    pub link_access: Option<String>,
    pub access: Vec<AccessEntry>,
}

struct ItemRow {
    name: String,
    owner_id: String,
    folder_id: Option<String>,
    created_at: String,
    updated_at: Option<String>,
    last_edited_by: Option<String>,
    public_role: Option<String>,
    size_bytes: i64,
    mime_type: Option<String>,
}

fn not_found() -> ApiError {
    (StatusCode::NOT_FOUND, "Item not found".to_string())
}

async fn document_row(state: &AppState, id: &str) -> Option<ItemRow> {
    let (name, owner_id, folder_id, created_at, updated_at, last_edited_by, public_role, size_bytes) =
        sqlx::query_as::<_, (String, String, Option<String>, String, String, Option<String>, Option<String>, i64)>(sql(
            "SELECT title, owner_id, folder_id, created_at, updated_at, last_edited_by, public_role, \
             CAST(COALESCE(LENGTH(content), 0) AS BIGINT) FROM documents WHERE id = $1",
        ))
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .ok()??;
    Some(ItemRow { name, owner_id, folder_id, created_at, updated_at: Some(updated_at), last_edited_by, public_role, size_bytes, mime_type: None })
}

async fn project_row(state: &AppState, id: &str) -> Option<ItemRow> {
    let (name, owner_id, folder_id, created_at, updated_at, last_edited_by, public_role, size_bytes) =
        sqlx::query_as::<_, (String, String, Option<String>, String, String, Option<String>, Option<String>, i64)>(sql(
            "SELECT p.name, p.owner_id, p.folder_id, p.created_at, p.updated_at, p.last_edited_by, p.public_role, \
             CAST(COALESCE((SELECT SUM(LENGTH(f.content)) FROM project_files f WHERE f.project_id = p.id), 0) AS BIGINT) \
             FROM projects p WHERE p.id = $1",
        ))
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .ok()??;
    Some(ItemRow { name, owner_id, folder_id, created_at, updated_at: Some(updated_at), last_edited_by, public_role, size_bytes, mime_type: None })
}

async fn file_row(state: &AppState, id: &str) -> Option<(ItemRow, Option<String>)> {
    let (name, owner_id, folder_id, document_id, created_at, updated_at, last_edited_by, size_bytes, mime_type) =
        sqlx::query_as::<_, (String, String, Option<String>, Option<String>, String, Option<String>, Option<String>, i64, String)>(sql(
            "SELECT name, owner_id, folder_id, document_id, created_at, updated_at, last_edited_by, size_bytes, mime_type \
             FROM files WHERE id = $1",
        ))
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .ok()??;
    let row = ItemRow { name, owner_id, folder_id, created_at, updated_at, last_edited_by, public_role: None, size_bytes, mime_type: Some(mime_type) };
    Some((row, document_id))
}

async fn folder_row(state: &AppState, id: &str) -> Option<ItemRow> {
    let (name, owner_id, parent_id, created_at, size_bytes) =
        sqlx::query_as::<_, (String, String, Option<String>, String, i64)>(sql(
            "WITH RECURSIVE tree(id) AS ( \
             SELECT id FROM folders WHERE id = $1 \
             UNION ALL SELECT f.id FROM folders f INNER JOIN tree t ON f.parent_id = t.id \
             ) SELECT fo.name, fo.owner_id, fo.parent_id, fo.created_at, CAST( \
             COALESCE((SELECT SUM(LENGTH(d.content)) FROM documents d WHERE d.folder_id IN (SELECT id FROM tree)), 0) \
             + COALESCE((SELECT SUM(fl.size_bytes) FROM files fl WHERE fl.folder_id IN (SELECT id FROM tree)), 0) \
             + COALESCE((SELECT SUM(LENGTH(pf.content)) FROM project_files pf INNER JOIN projects p ON p.id = pf.project_id \
             WHERE p.folder_id IN (SELECT id FROM tree)), 0) AS BIGINT) \
             FROM folders fo WHERE fo.id = $2",
        ))
        .bind(id)
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .ok()??;
    Some(ItemRow { name, owner_id, folder_id: parent_id, created_at, updated_at: None, last_edited_by: None, public_role: None, size_bytes, mime_type: None })
}

async fn load_item(state: &AppState, kind: &str, id: &str, user_id: &str) -> Result<(ItemRow, String, Option<ShareKind>), ApiError> {
    let user = Some(user_id.to_string());
    match kind {
        "document" => {
            let (_, role) = document_role(state, id, &user).await.ok_or_else(not_found)?;
            Ok((document_row(state, id).await.ok_or_else(not_found)?, role.as_str().to_string(), Some(ShareKind::Document)))
        }
        "project" => {
            let (_, role) = crate::drive::projects::project_role(state, id, &user).await.ok_or_else(not_found)?;
            Ok((project_row(state, id).await.ok_or_else(not_found)?, role, Some(ShareKind::Project)))
        }
        "folder" => {
            let role = folder_role(state, id, &user).await.ok_or_else(not_found)?;
            Ok((folder_row(state, id).await.ok_or_else(not_found)?, role.as_str().to_string(), Some(ShareKind::Folder)))
        }
        "file" => {
            let (row, document_id) = file_row(state, id).await.ok_or_else(not_found)?;
            let role = if row.owner_id == user_id {
                "owner".to_string()
            } else if crate::teams::content_role(state, &row.owner_id, user_id).await.is_some()
                || crate::drive::files::can_read_shared_file(state, id, &row.owner_id, &document_id, &row.folder_id, user_id).await
            {
                "viewer".to_string()
            } else {
                return Err(not_found());
            };
            Ok((row, role, None))
        }
        _ => Err((StatusCode::BAD_REQUEST, "Unknown item type".to_string())),
    }
}

pub async fn get_item_info(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path((kind, id)): Path<(String, String)>,
) -> Result<Json<ItemInfo>, ApiError> {
    let user_id = jar
        .get("session_user_id")
        .map(|cookie| cookie.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;
    let (row, your_role, share_kind) = load_item(&state, &kind, &id, &user_id).await?;

    let owner = person(&state, &row.owner_id).await.ok_or_else(not_found)?;
    let user = Some(user_id.clone());
    let location = match row.folder_id.as_deref() {
        None => Some(Vec::new()),
        Some(folder_id) if folder_role(&state, folder_id, &user).await.is_some() => Some(folder_path(&state, folder_id).await),
        Some(_) => None,
    };
    let last_edited_by = match row.last_edited_by.as_deref() {
        Some(editor_id) => person(&state, editor_id).await,
        None => None,
    };
    let access = who_has_access(&state, &owner, share_kind, &id, row.folder_id.as_deref()).await;

    Ok(Json(ItemInfo {
        kind,
        name: row.name,
        owner,
        your_role,
        size_bytes: row.size_bytes,
        location,
        created_at: row.created_at,
        updated_at: row.updated_at,
        last_edited_by,
        mime_type: row.mime_type,
        link_access: row.public_role.filter(|role| !role.is_empty()),
        access,
    }))
}

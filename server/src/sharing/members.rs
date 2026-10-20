use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use uuid::Uuid;

use crate::{
    sharing::access::{self, Resource},
    models::{CollaboratorView, InviteRequest},
    AppState,
};
use crate::db::{sql, sql_owned};

type ApiError = (StatusCode, String);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShareKind {
    Document,
    Project,
    Folder,
}

impl ShareKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "document" => Some(Self::Document),
            "project" => Some(Self::Project),
            "folder" => Some(Self::Folder),
            _ => None,
        }
    }

    pub fn item_table(self) -> &'static str {
        match self {
            Self::Document => "documents",
            Self::Project => "projects",
            Self::Folder => "folders",
        }
    }

    pub fn collaborator_table(self) -> &'static str {
        match self {
            Self::Document => "collaborators",
            Self::Project => "project_collaborators",
            Self::Folder => "folder_collaborators",
        }
    }

    pub fn item_column(self) -> &'static str {
        match self {
            Self::Document => "document_id",
            Self::Project => "project_id",
            Self::Folder => "folder_id",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Project => "project",
            Self::Folder => "folder",
        }
    }
}

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn session_user(jar: &SignedCookieJar) -> Result<String, ApiError> {
    jar.get("session_user_id")
        .map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))
}

pub async fn owner_of(state: &AppState, kind: ShareKind, item_id: &str) -> Result<String, ApiError> {
    let statement = format!("SELECT owner_id FROM {} WHERE id = $1", kind.item_table());
    sqlx::query_as::<_, (String,)>(sql_owned(statement))
        .bind(item_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .map(|(owner_id,)| owner_id)
        .ok_or((StatusCode::NOT_FOUND, format!("The {} was not found", kind.label())))
}

async fn require_owner(state: &AppState, kind: ShareKind, item_id: &str, user_id: &str) -> Result<(), ApiError> {
    if owner_of(state, kind, item_id).await? == user_id {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, format!("Only the owner can manage who has access to this {}", kind.label())))
    }
}

async fn has_access(state: &AppState, kind: ShareKind, item_id: &str, user_id: &str) -> Result<bool, ApiError> {
    let user = Some(user_id.to_string());
    if matches!(kind, ShareKind::Folder) {
        return Ok(access::folder_role(state, item_id, &user).await.is_some());
    }

    let direct_share = format!(
        "SELECT CAST(COUNT(*) AS BIGINT) FROM {} WHERE {} = $1 AND user_id = $2",
        kind.collaborator_table(),
        kind.item_column()
    );
    let (share_count,) = sqlx::query_as::<_, (i64,)>(sql_owned(direct_share))
        .bind(item_id)
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    if share_count > 0 {
        return Ok(true);
    }

    let item = format!("SELECT owner_id, folder_id FROM {} WHERE id = $1", kind.item_table());
    let Some((owner_id, folder_id)) = sqlx::query_as::<_, (String, Option<String>)>(sql_owned(item))
        .bind(item_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
    else {
        return Ok(false);
    };

    Ok(owner_id == user_id || access::inherited_role(state, &folder_id, &owner_id, &user).await.is_some())
}

pub async fn revalidate(state: &AppState, kind: ShareKind, item_id: &str, owner_id: &str) {
    match kind {
        ShareKind::Document => access::revalidate_resource(state, Resource::Document(item_id.to_string())).await,
        ShareKind::Project => access::revalidate_resource(state, Resource::Project(item_id.to_string())).await,
        ShareKind::Folder => {
            let documents = crate::drive::folders::items_in_folder_tree(state, "documents", item_id, owner_id).await.unwrap_or_default();
            let projects = crate::drive::folders::items_in_folder_tree(state, "projects", item_id, owner_id).await.unwrap_or_default();
            for document_id in documents {
                access::revalidate_resource(state, Resource::Document(document_id)).await;
            }
            for project_id in projects {
                access::revalidate_resource(state, Resource::Project(project_id)).await;
            }
        }
    }
}

async fn list(state: &AppState, kind: ShareKind, item_id: &str, jar: &SignedCookieJar) -> Result<Json<Vec<CollaboratorView>>, ApiError> {
    let user_id = session_user(jar)?;
    if !has_access(state, kind, item_id, &user_id).await? {
        return Err(access::forbidden());
    }

    let statement = format!(
        "SELECT c.id, c.user_id, u.username, u.email, c.role, c.created_at \
         FROM {} c INNER JOIN users u ON u.id = c.user_id \
         WHERE c.{} = $1 ORDER BY c.created_at ASC",
        kind.collaborator_table(),
        kind.item_column()
    );
    let collaborators = sqlx::query_as::<_, CollaboratorView>(sql_owned(statement))
        .bind(item_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;

    Ok(Json(collaborators))
}

async fn invite(
    state: &AppState,
    kind: ShareKind,
    item_id: &str,
    jar: &SignedCookieJar,
    payload: InviteRequest,
) -> Result<Json<CollaboratorView>, ApiError> {
    let inviter_id = session_user(jar)?;
    require_owner(state, kind, item_id, &inviter_id).await?;

    if payload.role != "viewer" && payload.role != "editor" {
        return Err((StatusCode::BAD_REQUEST, "Role must be viewer or editor".to_string()));
    }

    let (invited_user_id,) = sqlx::query_as::<_, (String,)>(sql("SELECT id FROM users WHERE LOWER(email) = LOWER($1)"))
        .bind(payload.email.trim())
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "User with that email not found".to_string()))?;

    if invited_user_id == inviter_id {
        return Err((StatusCode::BAD_REQUEST, format!("You already own this {}", kind.label())));
    }

    let upsert = format!(
        "INSERT INTO {table} (id, {column}, user_id, role) VALUES ($1, $2, $3, $4) \
         ON CONFLICT ({column}, user_id) DO UPDATE SET role = excluded.role",
        table = kind.collaborator_table(),
        column = kind.item_column()
    );
    sqlx::query(sql_owned(upsert))
        .bind(Uuid::new_v4().to_string())
        .bind(item_id)
        .bind(&invited_user_id)
        .bind(&payload.role)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    revalidate(state, kind, item_id, &inviter_id).await;

    let lookup = format!(
        "SELECT c.id, c.user_id, u.username, u.email, c.role, c.created_at \
         FROM {} c INNER JOIN users u ON u.id = c.user_id WHERE c.{} = $1 AND c.user_id = $2",
        kind.collaborator_table(),
        kind.item_column()
    );
    let collaborator = sqlx::query_as::<_, CollaboratorView>(sql_owned(lookup))
        .bind(item_id)
        .bind(&invited_user_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;

    Ok(Json(collaborator))
}

async fn remove(state: &AppState, kind: ShareKind, item_id: &str, collaborator_id: &str, jar: &SignedCookieJar) -> Result<StatusCode, ApiError> {
    let user_id = session_user(jar)?;
    require_owner(state, kind, item_id, &user_id).await?;

    let statement = format!("DELETE FROM {} WHERE id = $1 AND {} = $2", kind.collaborator_table(), kind.item_column());
    let result = sqlx::query(sql_owned(statement))
        .bind(collaborator_id)
        .bind(item_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Collaborator not found".to_string()));
    }

    revalidate(state, kind, item_id, &user_id).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_document_collaborators(State(state): State<AppState>, Path(id): Path<String>, jar: SignedCookieJar) -> Result<Json<Vec<CollaboratorView>>, ApiError> {
    list(&state, ShareKind::Document, &id, &jar).await
}

pub async fn invite_document_collaborator(State(state): State<AppState>, Path(id): Path<String>, jar: SignedCookieJar, Json(payload): Json<InviteRequest>) -> Result<Json<CollaboratorView>, ApiError> {
    invite(&state, ShareKind::Document, &id, &jar, payload).await
}

pub async fn remove_document_collaborator(State(state): State<AppState>, Path((id, collaborator_id)): Path<(String, String)>, jar: SignedCookieJar) -> Result<StatusCode, ApiError> {
    remove(&state, ShareKind::Document, &id, &collaborator_id, &jar).await
}

pub async fn list_project_collaborators(State(state): State<AppState>, Path(id): Path<String>, jar: SignedCookieJar) -> Result<Json<Vec<CollaboratorView>>, ApiError> {
    list(&state, ShareKind::Project, &id, &jar).await
}

pub async fn invite_project_collaborator(State(state): State<AppState>, Path(id): Path<String>, jar: SignedCookieJar, Json(payload): Json<InviteRequest>) -> Result<Json<CollaboratorView>, ApiError> {
    invite(&state, ShareKind::Project, &id, &jar, payload).await
}

pub async fn remove_project_collaborator(State(state): State<AppState>, Path((id, collaborator_id)): Path<(String, String)>, jar: SignedCookieJar) -> Result<StatusCode, ApiError> {
    remove(&state, ShareKind::Project, &id, &collaborator_id, &jar).await
}

pub async fn list_folder_collaborators(State(state): State<AppState>, Path(id): Path<String>, jar: SignedCookieJar) -> Result<Json<Vec<CollaboratorView>>, ApiError> {
    list(&state, ShareKind::Folder, &id, &jar).await
}

pub async fn invite_folder_collaborator(State(state): State<AppState>, Path(id): Path<String>, jar: SignedCookieJar, Json(payload): Json<InviteRequest>) -> Result<Json<CollaboratorView>, ApiError> {
    invite(&state, ShareKind::Folder, &id, &jar, payload).await
}

pub async fn remove_folder_collaborator(State(state): State<AppState>, Path((id, collaborator_id)): Path<(String, String)>, jar: SignedCookieJar) -> Result<StatusCode, ApiError> {
    remove(&state, ShareKind::Folder, &id, &collaborator_id, &jar).await
}

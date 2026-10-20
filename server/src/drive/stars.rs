use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Serialize;

use crate::db::sql_owned;
use crate::drive::projects::project_role;
use crate::models::{Document, Project};
use crate::sharing::access::document_role;
use crate::AppState;

type ApiError = (StatusCode, String);

#[derive(Clone, Copy)]
enum StarKind {
    Document,
    Project,
}

impl StarKind {
    fn parse(kind: &str) -> Result<Self, ApiError> {
        match kind {
            "document" => Ok(Self::Document),
            "project" => Ok(Self::Project),
            _ => Err((StatusCode::BAD_REQUEST, "Only documents and projects can be starred".to_string())),
        }
    }

    fn table(self) -> &'static str {
        match self {
            Self::Document => "document_stars",
            Self::Project => "project_stars",
        }
    }

    fn item_column(self) -> &'static str {
        match self {
            Self::Document => "document_id",
            Self::Project => "project_id",
        }
    }
}

#[derive(Serialize)]
pub struct StarredItems {
    pub documents: Vec<Document>,
    pub projects: Vec<Project>,
}

fn internal_error(error: impl ToString) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn session_user_id(jar: &SignedCookieJar) -> Result<String, ApiError> {
    jar.get("session_user_id")
        .map(|cookie| cookie.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))
}

async fn starred_ids(state: &AppState, kind: StarKind, user_id: &str) -> Result<Vec<String>, ApiError> {
    let query = format!(
        "SELECT {column} FROM {table} WHERE user_id = $1 ORDER BY created_at DESC",
        column = kind.item_column(),
        table = kind.table()
    );
    let rows = sqlx::query_as::<_, (String,)>(sql_owned(query))
        .bind(user_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

async fn has_access(state: &AppState, kind: StarKind, item_id: &str, user_id: &str) -> bool {
    let user = Some(user_id.to_string());
    match kind {
        StarKind::Document => document_role(state, item_id, &user).await.is_some(),
        StarKind::Project => project_role(state, item_id, &user).await.is_some(),
    }
}

async fn remove_star(state: &AppState, kind: StarKind, item_id: &str, user_id: &str) -> Result<(), ApiError> {
    let query = format!(
        "DELETE FROM {table} WHERE user_id = $1 AND {column} = $2",
        table = kind.table(),
        column = kind.item_column()
    );
    sqlx::query(sql_owned(query))
        .bind(user_id)
        .bind(item_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(())
}

pub async fn starred_items(state: &AppState, user_id: &str) -> Result<StarredItems, ApiError> {
    let user = Some(user_id.to_string());

    let mut documents = Vec::new();
    for id in starred_ids(state, StarKind::Document, user_id).await? {
        if let Some((mut document, role)) = document_role(state, &id, &user).await {
            document.effective_role = Some(role.as_str().to_string());
            documents.push(document);
        }
    }

    let mut projects = Vec::new();
    for id in starred_ids(state, StarKind::Project, user_id).await? {
        if let Some((mut project, role)) = project_role(state, &id, &user).await {
            project.effective_role = Some(role);
            projects.push(project);
        }
    }

    Ok(StarredItems { documents, projects })
}

pub async fn list_starred(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<StarredItems>, ApiError> {
    let user_id = session_user_id(&jar)?;
    Ok(Json(starred_items(&state, &user_id).await?))
}

pub async fn star_item(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path((kind, item_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let user_id = session_user_id(&jar)?;
    let kind = StarKind::parse(&kind)?;
    if !has_access(&state, kind, &item_id, &user_id).await {
        return Err((StatusCode::NOT_FOUND, "Item not found".to_string()));
    }

    remove_star(&state, kind, &item_id, &user_id).await?;
    let query = format!(
        "INSERT INTO {table} (user_id, {column}) VALUES ($1, $2)",
        table = kind.table(),
        column = kind.item_column()
    );
    sqlx::query(sql_owned(query))
        .bind(&user_id)
        .bind(&item_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn unstar_item(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path((kind, item_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let user_id = session_user_id(&jar)?;
    let kind = StarKind::parse(&kind)?;
    remove_star(&state, kind, &item_id, &user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

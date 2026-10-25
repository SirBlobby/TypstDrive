use std::collections::HashMap;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use super::import::{import_repository, preview_import, ImportPreview};
use super::links::{delete_link_rows, link_from_request, save_new_link, GitLink, LinkRequest};
use super::locks::lock_project;
use super::policy::load_policy;
use super::{internal_error, remote, ApiError};
use crate::accounts::signed_in::SignedInUser;
use crate::db::sql;
use crate::desktop::devices::{notify_devices, DeviceEvent};
use crate::models::Project;
use crate::sharing::access::require_owned_folder;
use crate::AppState;

const MAX_NAME_LENGTH: usize = 200;

#[derive(Deserialize)]
pub struct NewProjectRequest {
    pub name: String,
    pub folder_id: Option<String>,
    #[serde(flatten)]
    pub link: LinkRequest,
}

fn clean_name(name: &str) -> Result<String, ApiError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME_LENGTH {
        return Err((StatusCode::BAD_REQUEST, "Give the project a name of up to 200 characters".to_string()));
    }
    Ok(name.to_string())
}

async fn load_project(state: &AppState, project_id: &str) -> Result<Project, ApiError> {
    sqlx::query_as::<_, Project>(sql("SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at FROM projects WHERE id = $1"))
        .bind(project_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)
}

async fn insert_project(state: &AppState, project_id: &str, user_id: &str, folder_id: &Option<String>, name: &str) -> Result<Project, ApiError> {
    sqlx::query(sql("INSERT INTO projects (id, owner_id, folder_id, name, entrypoint) VALUES ($1, $2, $3, $4, 'main.typ')"))
        .bind(project_id)
        .bind(user_id)
        .bind(folder_id)
        .bind(name)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    load_project(state, project_id).await
}

async fn delete_project(state: &AppState, project_id: &str) {
    let _ = delete_link_rows(state, project_id).await;
    let _ = sqlx::query(sql("DELETE FROM project_files WHERE project_id = $1")).bind(project_id).execute(&state.db).await;
    let _ = sqlx::query(sql("DELETE FROM projects WHERE id = $1")).bind(project_id).execute(&state.db).await;
    remote::forget_cache(project_id);
}

async fn link_and_import(state: &AppState, project: &Project, link: &GitLink, user_id: &str) -> Result<(), ApiError> {
    let (preview, entries) = preview_import(state, &project.owner_id, &HashMap::new(), link, true).await?;
    save_new_link(state, link, user_id).await?;
    import_repository(state, project, link, &preview.commit, entries, true, user_id).await
}

pub async fn preview_new_project(
    State(state): State<AppState>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<LinkRequest>,
) -> Result<Json<ImportPreview>, ApiError> {
    let cache_name = format!("preview-{}", Uuid::new_v4());
    let link = link_from_request(&cache_name, &payload)?;
    load_policy(&state).await.check_provider(link.provider)?;
    let result = preview_import(&state, &user_id, &HashMap::new(), &link, true).await;
    remote::forget_cache(&cache_name);
    Ok(Json(result?.0))
}

pub async fn import_new_project(
    State(state): State<AppState>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<NewProjectRequest>,
) -> Result<Json<Project>, ApiError> {
    let name = clean_name(&payload.name)?;
    let folder_id = payload.folder_id.filter(|folder_id| !folder_id.is_empty());
    require_owned_folder(&state, &folder_id, &user_id).await?;

    let project_id = Uuid::new_v4().to_string();
    let link = link_from_request(&project_id, &payload.link)?;
    load_policy(&state).await.check_provider(link.provider)?;
    let _guard = lock_project(&project_id)?;

    let project = insert_project(&state, &project_id, &user_id, &folder_id, &name).await?;
    if let Err(error) = link_and_import(&state, &project, &link, &user_id).await {
        delete_project(&state, &project_id).await;
        return Err(error);
    }

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;
    Ok(Json(load_project(&state, &project_id).await?))
}

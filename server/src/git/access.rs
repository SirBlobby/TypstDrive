use axum::http::StatusCode;

use super::ApiError;
use crate::drive::projects::project_role;
use crate::models::Project;
use crate::AppState;

pub struct ProjectAccess {
    pub project: Project,
    pub is_owner: bool,
}

impl ProjectAccess {
    pub fn require_owner(&self) -> Result<(), ApiError> {
        if self.is_owner {
            Ok(())
        } else {
            Err((StatusCode::FORBIDDEN, "Only the project owner can change the Git link".to_string()))
        }
    }
}

pub async fn editable_project(state: &AppState, project_id: &str, user_id: &str) -> Result<ProjectAccess, ApiError> {
    let (project, role) = project_role(state, project_id, &Some(user_id.to_string()))
        .await
        .ok_or((StatusCode::NOT_FOUND, "Project not found".to_string()))?;
    match role.as_str() {
        "owner" => Ok(ProjectAccess { project, is_owner: true }),
        "editor" => Ok(ProjectAccess { project, is_owner: false }),
        _ => Err((StatusCode::FORBIDDEN, "You need edit access to sync this project with Git".to_string())),
    }
}

use axum::http::StatusCode;
use serde::Serialize;

use crate::db::sql;
use crate::realtime::live_documents::{document_room, project_file_room};
use crate::sharing::access::{self, AccessRole};
use crate::AppState;

type ApiError = (StatusCode, String);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReviewKind {
    Document,
    ProjectFile,
}

impl ReviewKind {
    pub fn comments_table(self) -> &'static str {
        match self {
            Self::Document => "comments",
            Self::ProjectFile => "project_comments",
        }
    }

    pub fn suggestions_table(self) -> &'static str {
        match self {
            Self::Document => "document_suggestions",
            Self::ProjectFile => "project_suggestions",
        }
    }

    pub fn target_column(self) -> &'static str {
        match self {
            Self::Document => "document_id",
            Self::ProjectFile => "file_id",
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(untagged)]
pub enum ReviewTarget {
    Document { document_id: String },
    ProjectFile { file_id: String },
}

impl ReviewTarget {
    pub fn new(kind: ReviewKind, target_id: String) -> Self {
        match kind {
            ReviewKind::Document => Self::Document { document_id: target_id },
            ReviewKind::ProjectFile => Self::ProjectFile { file_id: target_id },
        }
    }

    pub fn kind(&self) -> ReviewKind {
        match self {
            Self::Document { .. } => ReviewKind::Document,
            Self::ProjectFile { .. } => ReviewKind::ProjectFile,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Self::Document { document_id } => document_id,
            Self::ProjectFile { file_id } => file_id,
        }
    }
}

pub struct ReviewAccess {
    pub role: AccessRole,
    pub live_room: String,
}

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn file_not_found() -> ApiError {
    (StatusCode::NOT_FOUND, "File not found".to_string())
}

async fn project_of_file(state: &AppState, file_id: &str) -> Result<String, ApiError> {
    sqlx::query_as::<_, (String,)>(sql("SELECT project_id FROM project_files WHERE id = $1"))
        .bind(file_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .map(|(project_id,)| project_id)
        .ok_or_else(file_not_found)
}

pub async fn project_role(state: &AppState, project_id: &str, user_id: &str) -> Result<AccessRole, ApiError> {
    access::project_access(state, project_id, &Some(user_id.to_string()))
        .await
        .map(|(_, role)| role)
        .ok_or_else(access::forbidden)
}

pub async fn project_file_target(state: &AppState, project_id: &str, file_id: &str) -> Result<ReviewTarget, ApiError> {
    sqlx::query_as::<_, (String,)>(sql("SELECT id FROM project_files WHERE id = $1 AND project_id = $2 AND kind = 'text'"))
        .bind(file_id)
        .bind(project_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .ok_or_else(file_not_found)?;
    Ok(ReviewTarget::ProjectFile { file_id: file_id.to_string() })
}

pub async fn review_access(state: &AppState, target: &ReviewTarget, user_id: &str) -> Result<ReviewAccess, ApiError> {
    match target {
        ReviewTarget::Document { document_id } => {
            let (_, role) = access::document_role(state, document_id, &Some(user_id.to_string()))
                .await
                .ok_or_else(access::forbidden)?;
            Ok(ReviewAccess { role, live_room: document_room(document_id) })
        }
        ReviewTarget::ProjectFile { file_id } => {
            let project_id = project_of_file(state, file_id).await?;
            let role = project_role(state, &project_id, user_id).await?;
            Ok(ReviewAccess { role, live_room: project_file_room(&project_id, file_id) })
        }
    }
}

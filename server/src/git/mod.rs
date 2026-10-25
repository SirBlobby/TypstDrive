mod access;
mod file_rules;
mod import;
pub mod links;
mod local_files;
mod locks;
pub mod new_project;
pub mod policy;
pub mod pull;
pub mod push;
mod remote;
mod repo_settings;
mod snapshot;
pub mod status;
mod sync_plan;

use axum::http::StatusCode;

pub type ApiError = (StatusCode, String);

pub fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

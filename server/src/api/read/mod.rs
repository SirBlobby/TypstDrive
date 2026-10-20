pub mod documents;
pub mod files;
pub mod folders;
pub mod projects;
pub mod starred;
pub mod teams;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

fn database_error(error: sqlx::Error) -> Response {
    tracing::warn!("Public API query failed: {error}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
}

fn not_found(label: &str) -> Response {
    (StatusCode::NOT_FOUND, format!("{label} not found")).into_response()
}

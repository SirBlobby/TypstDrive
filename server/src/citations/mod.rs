pub mod bibtex;
pub mod csl;
pub mod entries;
pub mod format;
pub mod hayagriva;
pub mod lookup;
pub mod policy;
pub mod secrets;
pub mod sync;
pub mod targets;
pub mod zotero;
pub mod zotero_account;

use axum::http::StatusCode;

pub type ApiError = (StatusCode, String);

pub fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

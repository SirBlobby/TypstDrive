mod authors;
pub mod create;
mod files;
pub mod index;
pub mod thumbnails;

use std::time::Duration;

use axum::http::StatusCode;

pub type ApiError = (StatusCode, String);

const REGISTRY_URL: &str = "https://packages.typst.org";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

fn http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(REQUEST_TIMEOUT)
        .user_agent("TypstDrive (templates)")
        .build()
}

fn internal_error(error: impl std::fmt::Display) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn unreachable_registry() -> ApiError {
    (StatusCode::BAD_GATEWAY, "Could not reach Typst Universe. Try again in a moment.".to_string())
}

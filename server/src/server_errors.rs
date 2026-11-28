use axum::http::StatusCode;

pub const GENERIC_MESSAGE: &str = "Something went wrong on the server. Try again in a moment.";

pub fn hidden(error: impl ToString) -> (StatusCode, String) {
    tracing::error!("Internal error: {}", error.to_string());
    (StatusCode::INTERNAL_SERVER_ERROR, GENERIC_MESSAGE.to_string())
}

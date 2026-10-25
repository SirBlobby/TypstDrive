pub mod admin;
pub mod cache;
pub mod palette;
pub mod personal;
pub mod policy;
mod store;

use axum::http::StatusCode;

use palette::Theme;

pub type ApiError = (StatusCode, String);

pub fn internal_error(error: impl std::fmt::Display) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn ensure_exists(themes: &[Theme], id: &str) -> Result<(), ApiError> {
    if themes.iter().any(|theme| theme.id == id) {
        Ok(())
    } else {
        Err((StatusCode::NOT_FOUND, "Theme not found".to_string()))
    }
}

fn check_name_free(themes: &[Theme], name: &str, editing_id: Option<&str>) -> Result<(), ApiError> {
    let taken = themes
        .iter()
        .any(|theme| Some(theme.id.as_str()) != editing_id && theme.name.eq_ignore_ascii_case(name));
    if taken {
        Err((StatusCode::CONFLICT, format!("A theme named {name} already exists")))
    } else {
        Ok(())
    }
}

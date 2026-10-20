use axum::http::StatusCode;

const MAX_PATH_LENGTH: usize = 512;

fn invalid(message: &str) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, message.to_string())
}

pub fn clean_project_path(path: &str) -> Result<String, (StatusCode, String)> {
    let path = path.trim();
    if path.is_empty() {
        return Err(invalid("File path cannot be empty"));
    }
    if path.len() > MAX_PATH_LENGTH {
        return Err(invalid("File path is too long"));
    }
    if path.starts_with('/') || path.contains('\\') || path.chars().any(char::is_control) {
        return Err(invalid("File path must be relative and use forward slashes"));
    }
    let has_bad_segment = path
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == ".." || segment.trim() != segment);
    if has_bad_segment {
        return Err(invalid("File path cannot contain empty, '.' or '..' parts"));
    }
    Ok(path.to_string())
}

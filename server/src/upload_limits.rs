use axum::extract::DefaultBodyLimit;

const DEFAULT_UPLOAD_MB: usize = 25;
const JSON_OVERHEAD_BYTES: usize = 64 * 1024;

pub fn upload_bytes() -> usize {
    std::env::var("UPLOAD_MAX_MB")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|megabytes| *megabytes > 0)
        .unwrap_or(DEFAULT_UPLOAD_MB)
        .saturating_mul(1024 * 1024)
}

pub fn file_uploads() -> DefaultBodyLimit {
    DefaultBodyLimit::max(upload_bytes() + JSON_OVERHEAD_BYTES)
}

pub fn encoded_uploads() -> DefaultBodyLimit {
    DefaultBodyLimit::max(upload_bytes() / 3 * 4 + JSON_OVERHEAD_BYTES)
}

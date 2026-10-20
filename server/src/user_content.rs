use axum::http::header;
use axum::response::{IntoResponse, Response};

const INLINE_TYPES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "image/avif",
    "image/bmp",
    "image/x-icon",
    "image/vnd.microsoft.icon",
    "application/pdf",
    "text/plain",
    "text/csv",
];

const INLINE_PREFIXES: &[&str] = &["font/", "audio/", "video/"];

const SVG_TYPE: &str = "image/svg+xml";
const SANDBOX_POLICY: &str = "sandbox; default-src 'none'; img-src data:; style-src 'unsafe-inline'";

fn essence(mime_type: &str) -> String {
    mime_type.split(';').next().unwrap_or_default().trim().to_ascii_lowercase()
}

fn is_inline_type(mime_essence: &str) -> bool {
    INLINE_TYPES.contains(&mime_essence) || INLINE_PREFIXES.iter().any(|prefix| mime_essence.starts_with(prefix))
}

fn declared_type(mime_essence: &str, original: &str) -> String {
    let is_valid = !mime_essence.is_empty()
        && mime_essence.contains('/')
        && original.chars().all(|character| character.is_ascii_graphic() || character == ' ');
    if is_valid {
        original.trim().to_string()
    } else {
        "application/octet-stream".to_string()
    }
}

pub fn response(mime_type: &str, data: Vec<u8>) -> Response {
    let mime_essence = essence(mime_type);
    let content_type = declared_type(&mime_essence, mime_type);

    if is_inline_type(&mime_essence) {
        return (
            [
                (header::CONTENT_TYPE, content_type),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
            ],
            data,
        )
            .into_response();
    }

    if mime_essence == SVG_TYPE {
        return (
            [
                (header::CONTENT_TYPE, content_type),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
                (header::CONTENT_SECURITY_POLICY, SANDBOX_POLICY.to_string()),
            ],
            data,
        )
            .into_response();
    }

    (
        [
            (header::CONTENT_TYPE, content_type),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
            (header::CONTENT_SECURITY_POLICY, SANDBOX_POLICY.to_string()),
            (header::CONTENT_DISPOSITION, "attachment".to_string()),
        ],
        data,
    )
        .into_response()
}

pub fn text_response(text: String) -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        text,
    )
        .into_response()
}

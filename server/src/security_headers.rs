use std::sync::OnceLock;

use axum::extract::Request;
use axum::http::{header, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;

const STRICT_TRANSPORT: &str = "max-age=31536000";

fn serves_https() -> bool {
    static SERVES_HTTPS: OnceLock<bool> = OnceLock::new();
    *SERVES_HTTPS.get_or_init(crate::site_url::uses_https)
}

pub async fn add_security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.entry(header::X_CONTENT_TYPE_OPTIONS).or_insert(HeaderValue::from_static("nosniff"));
    headers.entry(header::REFERRER_POLICY).or_insert(HeaderValue::from_static("same-origin"));
    headers.entry(header::X_FRAME_OPTIONS).or_insert(HeaderValue::from_static("SAMEORIGIN"));
    if serves_https() {
        headers.entry(header::STRICT_TRANSPORT_SECURITY).or_insert(HeaderValue::from_static(STRICT_TRANSPORT));
    }
    response
}

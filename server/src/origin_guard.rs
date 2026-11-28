use axum::extract::Request;
use axum::http::{header, HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

fn authority(url: &str) -> Option<&str> {
    let rest = url.split_once("://")?.1;
    let authority = rest.split('/').next()?;
    (!authority.is_empty()).then_some(authority)
}

fn origin_matches(headers: &HeaderMap) -> bool {
    let Some(origin) = headers.get(header::ORIGIN).and_then(|value| value.to_str().ok()) else {
        return true;
    };
    let Some(origin_authority) = authority(origin) else {
        return false;
    };
    let host = headers.get(header::HOST).and_then(|value| value.to_str().ok());
    let public = crate::site_url::public_url();
    host.is_some_and(|host| host.eq_ignore_ascii_case(origin_authority))
        || public.as_deref().and_then(authority).is_some_and(|public| public.eq_ignore_ascii_case(origin_authority))
}

pub async fn same_origin_sockets(request: Request, next: Next) -> Response {
    let uses_cookies = request.headers().contains_key(header::COOKIE);
    if uses_cookies && !origin_matches(request.headers()) {
        return (StatusCode::FORBIDDEN, "Connections from other sites are not allowed").into_response();
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::origin_matches;
    use axum::http::{header, HeaderMap, HeaderValue};

    fn headers(origin: Option<&'static str>, host: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_static(host));
        if let Some(origin) = origin {
            headers.insert(header::ORIGIN, HeaderValue::from_static(origin));
        }
        headers
    }

    #[test]
    fn accepts_same_host_and_missing_origin() {
        assert!(origin_matches(&headers(Some("https://drive.example.com"), "drive.example.com")));
        assert!(origin_matches(&headers(None, "drive.example.com")));
    }

    #[test]
    fn rejects_other_sites() {
        assert!(!origin_matches(&headers(Some("https://evil.example.com"), "drive.example.com")));
        assert!(!origin_matches(&headers(Some("null"), "drive.example.com")));
    }
}

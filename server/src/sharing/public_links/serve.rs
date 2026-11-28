use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};

use super::target::{self, record_view};
use super::{cache, Format};
use crate::AppState;

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "Public link not found").into_response()
}

fn pdf_response(slug: &str, fingerprint: &str, bytes: Vec<u8>) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(header::CONTENT_DISPOSITION, format!("inline; filename=\"{slug}.pdf\""))
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::ETAG, format!("\"{fingerprint}\""))
        .body(Body::from(bytes))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn not_modified(fingerprint: &str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_MODIFIED)
        .header(header::ETAG, format!("\"{fingerprint}\""))
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::NOT_MODIFIED.into_response())
}

fn matches_etag(headers: &HeaderMap, fingerprint: &str) -> bool {
    headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.split(',').any(|tag| tag.trim().trim_start_matches("W/") == format!("\"{fingerprint}\"")))
}

async fn fresh_response(state: &AppState, link_id: &str, slug: &str, headers: &HeaderMap) -> Option<Response> {
    let (fingerprint, bytes) = cache::fresh(link_id)?;
    record_view(state, link_id, Format::Pdf).await;
    if matches_etag(headers, &fingerprint) {
        return Some(not_modified(&fingerprint));
    }
    Some(pdf_response(slug, &fingerprint, bytes))
}

pub async fn serve_pdf(State(state): State<AppState>, Path(slug): Path<String>, headers: HeaderMap) -> Response {
    let Some(link) = target::find(&state, &slug, Format::Pdf).await else {
        return not_found();
    };
    let link_id = link.id.clone();
    let slug = link.slug.clone();

    if let Some(response) = fresh_response(&state, &link_id, &slug, &headers).await {
        return response;
    }
    let lock = cache::compile_lock(&link_id);
    let _compiling = lock.lock().await;
    if let Some(response) = fresh_response(&state, &link_id, &slug, &headers).await {
        return response;
    }

    let Some(input) = target::compile_input(&state, &link).await else {
        return not_found();
    };

    record_view(&state, &link_id, Format::Pdf).await;
    let fingerprint = cache::fingerprint(&input);
    if matches_etag(&headers, &fingerprint) {
        return not_modified(&fingerprint);
    }

    let previous = cache::latest(&link_id);
    if let Some((cached_fingerprint, bytes)) = &previous {
        if *cached_fingerprint == fingerprint {
            cache::touch(&link_id);
            return pdf_response(&slug, &fingerprint, bytes.clone());
        }
    }

    let compiled = if cache::has_failed(&link_id, &fingerprint) {
        None
    } else {
        Some(crate::compile::render::pdf(input).await)
    };
    match compiled {
        Some(Ok(bytes)) => {
            cache::remember(&link_id, &fingerprint, &bytes);
            pdf_response(&slug, &fingerprint, bytes)
        }
        failure => {
            if matches!(failure, Some(Err(_))) {
                cache::remember_failure(&link_id, &fingerprint);
            }
            unavailable(&slug, previous)
        }
    }
}

fn unavailable(slug: &str, previous: Option<(String, Vec<u8>)>) -> Response {
    match previous {
        Some((cached_fingerprint, bytes)) => pdf_response(slug, &cached_fingerprint, bytes),
        None => (StatusCode::UNPROCESSABLE_ENTITY, "This document has errors and cannot be compiled right now").into_response(),
    }
}

use std::collections::HashMap;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};

use super::cache;
use super::policy::load_policy;
use crate::compile::compiler::{run_blocking, ProjectInput};
use crate::compile::latex;
use crate::db::{sql, timestamp_now};
use crate::drive::versions::current_text;
use crate::models::Project;
use crate::sharing::access::{self, AccessRole};
use crate::AppState;

type LinkTarget = (String, Option<String>, Option<String>);

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "PDF link not found").into_response()
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

async fn document_input(state: &AppState, document_id: &str) -> Option<ProjectInput> {
    let (owner_id,) = sqlx::query_as::<_, (String,)>(sql("SELECT owner_id FROM documents WHERE id = $1"))
        .bind(document_id)
        .fetch_optional(&state.db)
        .await
        .ok()??;
    let (document, _) = access::document_role(state, document_id, &Some(owner_id)).await?;
    let text = current_text(state, &document.id).await.unwrap_or_default();
    let files = crate::drive::document_assets::compile_files(state, &document, AccessRole::Viewer, &None).await;
    Some(latex::document_input(&document, text, files))
}

async fn project_input(state: &AppState, project_id: &str) -> Option<ProjectInput> {
    let project = sqlx::query_as::<_, Project>(
        sql("SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at FROM projects WHERE id = $1"),
    )
    .bind(project_id)
    .fetch_optional(&state.db)
    .await
    .ok()??;
    let reader = crate::drive::projects::ProjectReader::Collaborator(None);
    Some(crate::drive::projects::assemble_project(state, &project, HashMap::new(), reader).await)
}

async fn record_view(state: &AppState, link_id: &str) {
    let _ = sqlx::query(sql("UPDATE pdf_links SET view_count = view_count + 1, last_viewed_at = $1 WHERE id = $2"))
        .bind(timestamp_now())
        .bind(link_id)
        .execute(&state.db)
        .await;
}

pub async fn serve_pdf(State(state): State<AppState>, Path(slug): Path<String>, headers: HeaderMap) -> Response {
    if !load_policy(&state).await.enabled {
        return not_found();
    }
    let slug = slug.strip_suffix(".pdf").unwrap_or(&slug).to_lowercase();

    let target = sqlx::query_as::<_, LinkTarget>(sql("SELECT id, document_id, project_id FROM pdf_links WHERE slug = $1"))
        .bind(&slug)
        .fetch_optional(&state.db)
        .await;
    let Ok(Some((link_id, document_id, project_id))) = target else {
        return not_found();
    };

    if let Some((fingerprint, bytes)) = cache::fresh(&link_id) {
        record_view(&state, &link_id).await;
        if matches_etag(&headers, &fingerprint) {
            return not_modified(&fingerprint);
        }
        return pdf_response(&slug, &fingerprint, bytes);
    }

    let input = match (document_id, project_id) {
        (Some(document_id), _) => document_input(&state, &document_id).await,
        (None, Some(project_id)) => project_input(&state, &project_id).await,
        (None, None) => None,
    };
    let Some(input) = input else {
        return not_found();
    };

    record_view(&state, &link_id).await;
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
        run_blocking(move |compiler| compiler.export_pdf(input)).await
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

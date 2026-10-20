use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use super::documents::DocumentSummary;
use super::projects::ProjectSummary;
use crate::api::authentication::{authenticate, ApiScope};
use crate::drive::stars::starred_items;
use crate::AppState;

#[derive(Serialize)]
pub struct StarredSummary {
    pub documents: Vec<DocumentSummary>,
    pub projects: Vec<ProjectSummary>,
}

pub async fn list_starred(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<StarredSummary>, Response> {
    let caller = authenticate(&state, &headers, ApiScope::Read).await?;
    let starred = starred_items(&state, &caller.user_id).await.map_err(IntoResponse::into_response)?;

    let documents = starred
        .documents
        .into_iter()
        .map(|document| DocumentSummary {
            id: document.id,
            title: document.title,
            folder_id: document.folder_id,
            created_at: document.created_at,
            updated_at: document.updated_at,
        })
        .collect();
    let projects = starred
        .projects
        .into_iter()
        .map(|project| ProjectSummary {
            id: project.id,
            name: project.name,
            entrypoint: project.entrypoint,
            folder_id: project.folder_id,
            created_at: project.created_at,
            updated_at: project.updated_at,
        })
        .collect();

    Ok(Json(StarredSummary { documents, projects }))
}

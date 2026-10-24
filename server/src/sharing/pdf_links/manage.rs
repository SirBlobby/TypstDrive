use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::policy::load_policy;
use super::{cache, clean_slug, internal_error, link_for_item, links_for_owner, suggested_slug, ApiError, PdfLinkKind, PdfLinkView};
use crate::db::sql;
use crate::sharing::access::{self, AccessRole};
use crate::AppState;

#[derive(Serialize)]
pub struct ItemPdfLink {
    pub enabled: bool,
    pub can_manage: bool,
    pub suggested_slug: String,
    pub link: Option<PdfLinkView>,
}

#[derive(Serialize)]
pub struct OwnPdfLinks {
    pub enabled: bool,
    pub max_links_per_user: i64,
    pub links: Vec<PdfLinkView>,
}

#[derive(Deserialize)]
pub struct SavePdfLinkRequest {
    pub slug: String,
}

struct ItemAccess {
    name: String,
    owner_id: String,
    is_owner: bool,
}

fn session_user(jar: &SignedCookieJar) -> Option<String> {
    jar.get("session_user_id").map(|c| c.value().to_string())
}

async fn item_access(state: &AppState, kind: PdfLinkKind, item_id: &str, user_id: &Option<String>) -> Result<ItemAccess, ApiError> {
    let not_found = || (StatusCode::NOT_FOUND, format!("{} not found", kind.as_str()));
    match kind {
        PdfLinkKind::Document => {
            let (document, role) = access::document_role(state, item_id, user_id).await.ok_or_else(not_found)?;
            Ok(ItemAccess { name: document.title, owner_id: document.owner_id, is_owner: role == AccessRole::Owner })
        }
        PdfLinkKind::Project => {
            let (project, role) = access::project_access(state, item_id, user_id).await.ok_or_else(not_found)?;
            Ok(ItemAccess { name: project.name, owner_id: project.owner_id, is_owner: role == AccessRole::Owner })
        }
    }
}

async fn owned_item(state: &AppState, jar: &SignedCookieJar, kind: &str, item_id: &str) -> Result<(PdfLinkKind, ItemAccess), ApiError> {
    let user_id = session_user(jar).ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;
    let kind = PdfLinkKind::parse(kind)?;
    let item = item_access(state, kind, item_id, &Some(user_id)).await?;
    if !item.is_owner {
        return Err((StatusCode::FORBIDDEN, "Only the owner can manage the public PDF link".to_string()));
    }
    Ok((kind, item))
}

async fn slug_taken(state: &AppState, slug: &str, except_id: Option<&str>) -> Result<bool, ApiError> {
    let existing = sqlx::query_as::<_, (String,)>(sql("SELECT id FROM pdf_links WHERE slug = $1"))
        .bind(slug)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(existing.is_some_and(|(id,)| Some(id.as_str()) != except_id))
}

pub async fn get_item_link(
    State(state): State<AppState>,
    Path((kind, item_id)): Path<(String, String)>,
    jar: SignedCookieJar,
) -> Result<Json<ItemPdfLink>, ApiError> {
    let kind = PdfLinkKind::parse(&kind)?;
    let item = item_access(&state, kind, &item_id, &session_user(&jar)).await?;
    Ok(Json(ItemPdfLink {
        enabled: load_policy(&state).await.enabled,
        can_manage: item.is_owner,
        suggested_slug: suggested_slug(&item.name),
        link: link_for_item(&state, kind, &item_id).await?,
    }))
}

pub async fn save_item_link(
    State(state): State<AppState>,
    Path((kind, item_id)): Path<(String, String)>,
    jar: SignedCookieJar,
    Json(payload): Json<SavePdfLinkRequest>,
) -> Result<Json<PdfLinkView>, ApiError> {
    let (kind, item) = owned_item(&state, &jar, &kind, &item_id).await?;
    let policy = load_policy(&state).await;
    policy.check_enabled()?;
    let slug = clean_slug(&payload.slug)?;
    let existing = link_for_item(&state, kind, &item_id).await?;

    if slug_taken(&state, &slug, existing.as_ref().map(|link| link.id.as_str())).await? {
        return Err((StatusCode::CONFLICT, "That short link is already taken".to_string()));
    }

    match existing {
        Some(link) => {
            sqlx::query(sql("UPDATE pdf_links SET slug = $1 WHERE id = $2"))
                .bind(&slug)
                .bind(&link.id)
                .execute(&state.db)
                .await
                .map_err(internal_error)?;
        }
        None => {
            policy.check_room(links_for_owner(&state, &item.owner_id).await?.len() as i64)?;
            let (document_id, project_id) = match kind {
                PdfLinkKind::Document => (Some(item_id.clone()), None),
                PdfLinkKind::Project => (None, Some(item_id.clone())),
            };
            sqlx::query(sql("INSERT INTO pdf_links (id, slug, document_id, project_id) VALUES ($1, $2, $3, $4)"))
                .bind(Uuid::new_v4().to_string())
                .bind(&slug)
                .bind(document_id)
                .bind(project_id)
                .execute(&state.db)
                .await
                .map_err(|_| (StatusCode::CONFLICT, "That short link is already taken".to_string()))?;
        }
    }

    link_for_item(&state, kind, &item_id)
        .await?
        .map(Json)
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "The link could not be saved".to_string()))
}

pub async fn delete_item_link(
    State(state): State<AppState>,
    Path((kind, item_id)): Path<(String, String)>,
    jar: SignedCookieJar,
) -> Result<StatusCode, ApiError> {
    let (kind, _) = owned_item(&state, &jar, &kind, &item_id).await?;
    if let Some(link) = link_for_item(&state, kind, &item_id).await? {
        sqlx::query(sql("DELETE FROM pdf_links WHERE id = $1"))
            .bind(&link.id)
            .execute(&state.db)
            .await
            .map_err(internal_error)?;
        cache::forget(&link.id);
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_own_links(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<OwnPdfLinks>, ApiError> {
    let user_id = session_user(&jar).ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;
    let policy = load_policy(&state).await;
    Ok(Json(OwnPdfLinks {
        enabled: policy.enabled,
        max_links_per_user: policy.max_links_per_user,
        links: links_for_owner(&state, &user_id).await?,
    }))
}

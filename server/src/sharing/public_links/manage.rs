use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::policy::{load_policy, PublicLinkPolicy};
use super::{cache, clean_slug, internal_error, link_for_item, links_for_owner, suggested_slug, ApiError, Format, LinkKind, PublicLinkView};
use crate::db::sql;
use crate::sharing::access::{self, AccessRole};
use crate::AppState;

#[derive(Serialize)]
pub struct ItemPublicLink {
    pub policy: PublicLinkPolicy,
    pub can_manage: bool,
    pub suggested_slug: String,
    pub link: Option<PublicLinkView>,
}

#[derive(Serialize)]
pub struct OwnPublicLinks {
    pub policy: PublicLinkPolicy,
    pub links: Vec<PublicLinkView>,
}

#[derive(Deserialize)]
pub struct SavePublicLinkRequest {
    pub slug: String,
    pub pdf_enabled: bool,
    pub slides_enabled: bool,
    pub speaker_enabled: bool,
    #[serde(default)]
    pub speaker_password: Option<String>,
}

impl SavePublicLinkRequest {
    fn wants(&self, format: Format) -> bool {
        match format {
            Format::Pdf => self.pdf_enabled,
            Format::Slides => self.slides_enabled,
            Format::Speaker => self.speaker_enabled,
        }
    }
}

struct ItemAccess {
    name: String,
    owner_id: String,
    is_owner: bool,
}

fn session_user(jar: &SignedCookieJar) -> Option<String> {
    jar.get("session_user_id").map(|c| c.value().to_string())
}

fn was_enabled(link: Option<&PublicLinkView>, format: Format) -> bool {
    link.is_some_and(|link| match format {
        Format::Pdf => link.pdf.enabled,
        Format::Slides => link.slides.enabled,
        Format::Speaker => link.speaker.enabled,
    })
}

async fn new_speaker_password_hash(payload: &SavePublicLinkRequest) -> Result<Option<String>, ApiError> {
    let Some(password) = payload.speaker_password.as_deref().filter(|password| !password.is_empty()) else {
        return Ok(None);
    };
    crate::accounts::passwords::check_new_password(password, crate::accounts::passwords::DEFAULT_MIN_PASSWORD_LENGTH)?;
    crate::accounts::passwords::hash_password(password)
        .await
        .map(Some)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))
}

async fn item_access(state: &AppState, kind: LinkKind, item_id: &str, user_id: &Option<String>) -> Result<ItemAccess, ApiError> {
    let not_found = || (StatusCode::NOT_FOUND, format!("{} not found", kind.as_str()));
    match kind {
        LinkKind::Document => {
            let (document, role) = access::document_role(state, item_id, user_id).await.ok_or_else(not_found)?;
            Ok(ItemAccess { name: document.title, owner_id: document.owner_id, is_owner: role == AccessRole::Owner })
        }
        LinkKind::Project => {
            let (project, role) = access::project_access(state, item_id, user_id).await.ok_or_else(not_found)?;
            Ok(ItemAccess { name: project.name, owner_id: project.owner_id, is_owner: role == AccessRole::Owner })
        }
    }
}

async fn owned_item(state: &AppState, jar: &SignedCookieJar, kind: &str, item_id: &str) -> Result<(LinkKind, ItemAccess), ApiError> {
    let user_id = session_user(jar).ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;
    let kind = LinkKind::parse(kind)?;
    let item = item_access(state, kind, item_id, &Some(user_id)).await?;
    if !item.is_owner {
        return Err((StatusCode::FORBIDDEN, "Only the owner can manage the public link".to_string()));
    }
    Ok((kind, item))
}

async fn slug_taken(state: &AppState, slug: &str, except_id: Option<&str>) -> Result<bool, ApiError> {
    let existing = sqlx::query_as::<_, (String,)>(sql("SELECT id FROM public_links WHERE slug = $1"))
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
) -> Result<Json<ItemPublicLink>, ApiError> {
    let kind = LinkKind::parse(&kind)?;
    let item = item_access(&state, kind, &item_id, &session_user(&jar)).await?;
    Ok(Json(ItemPublicLink {
        policy: load_policy(&state).await,
        can_manage: item.is_owner,
        suggested_slug: suggested_slug(&item.name),
        link: link_for_item(&state, kind, &item_id).await?,
    }))
}

async fn insert_link(state: &AppState, kind: LinkKind, item_id: &str, slug: &str) -> Result<String, ApiError> {
    let (document_id, project_id) = match kind {
        LinkKind::Document => (Some(item_id.to_string()), None),
        LinkKind::Project => (None, Some(item_id.to_string())),
    };
    let link_id = Uuid::new_v4().to_string();
    sqlx::query(sql("INSERT INTO public_links (id, slug, document_id, project_id) VALUES ($1, $2, $3, $4)"))
        .bind(&link_id)
        .bind(slug)
        .bind(document_id)
        .bind(project_id)
        .execute(&state.db)
        .await
        .map_err(|_| (StatusCode::CONFLICT, "That link name is already taken".to_string()))?;
    Ok(link_id)
}

pub async fn save_item_link(
    State(state): State<AppState>,
    Path((kind, item_id)): Path<(String, String)>,
    jar: SignedCookieJar,
    Json(payload): Json<SavePublicLinkRequest>,
) -> Result<Json<PublicLinkView>, ApiError> {
    let (kind, item) = owned_item(&state, &jar, &kind, &item_id).await?;
    let policy = load_policy(&state).await;
    policy.check_enabled()?;
    let slug = clean_slug(&payload.slug)?;
    let existing = link_for_item(&state, kind, &item_id).await?;

    if !payload.pdf_enabled && !payload.slides_enabled && !payload.speaker_enabled {
        return Err((StatusCode::BAD_REQUEST, "Turn on at least one format, or remove the link".to_string()));
    }
    for format in [Format::Pdf, Format::Slides, Format::Speaker] {
        if payload.wants(format) && !was_enabled(existing.as_ref(), format) {
            policy.check_format(format)?;
        }
    }
    let new_password_hash = new_speaker_password_hash(&payload).await?;
    let has_password = new_password_hash.is_some() || existing.as_ref().is_some_and(|link| link.has_speaker_password);
    if payload.speaker_enabled && !has_password {
        return Err((StatusCode::BAD_REQUEST, "Set a password for the speaker view".to_string()));
    }
    if slug_taken(&state, &slug, existing.as_ref().map(|link| link.id.as_str())).await? {
        return Err((StatusCode::CONFLICT, "That link name is already taken".to_string()));
    }

    let link_id = match &existing {
        Some(link) => link.id.clone(),
        None => {
            policy.check_room(links_for_owner(&state, &item.owner_id).await?.len() as i64)?;
            insert_link(&state, kind, &item_id, &slug).await?
        }
    };

    let flag = |enabled: bool| if enabled { 1i64 } else { 0i64 };
    sqlx::query(sql(
        "UPDATE public_links SET slug = $1, pdf_enabled = $2, slides_enabled = $3, speaker_enabled = $4, \
         speaker_password_hash = COALESCE($5, speaker_password_hash) WHERE id = $6",
    ))
    .bind(&slug)
    .bind(flag(payload.pdf_enabled))
    .bind(flag(payload.slides_enabled))
    .bind(flag(payload.speaker_enabled))
    .bind(new_password_hash)
    .bind(&link_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::CONFLICT, "That link name is already taken".to_string()))?;

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
        sqlx::query(sql("DELETE FROM public_links WHERE id = $1"))
            .bind(&link.id)
            .execute(&state.db)
            .await
            .map_err(internal_error)?;
        cache::forget(&link.id);
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_own_links(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<OwnPublicLinks>, ApiError> {
    let user_id = session_user(&jar).ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;
    Ok(Json(OwnPublicLinks { policy: load_policy(&state).await, links: links_for_owner(&state, &user_id).await? }))
}

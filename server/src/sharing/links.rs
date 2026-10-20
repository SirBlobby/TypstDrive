use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use axum_extra::extract::cookie::{Cookie, Key, SameSite, SignedCookieJar};
use serde::{Deserialize, Serialize};

use crate::accounts::client_info::ClientInfo;
use crate::accounts::passwords;
use crate::db::{sql_owned, timestamp_now};
use crate::sharing::access::{self, AccessRole, Resource};
use crate::AppState;

const GRANTS_COOKIE: &str = "typstdrive_links";
const MAX_GRANTS: usize = 50;
const UNLOCK_SCOPE: &str = "share-link";

type ApiError = (StatusCode, String);

tokio::task_local! {
    static LINK_GRANTS: Vec<String>;
}

#[derive(Clone, Copy)]
enum LinkKind {
    Document,
    Project,
}

fn invalid_role_message(kind: LinkKind) -> &'static str {
    match kind {
        LinkKind::Document => "Role must be viewer, suggester, editor or none",
        LinkKind::Project => "Role must be viewer, editor or none",
    }
}

impl LinkKind {
    fn parse(value: &str) -> Result<Self, ApiError> {
        match value {
            "document" => Ok(Self::Document),
            "project" => Ok(Self::Project),
            _ => Err((StatusCode::BAD_REQUEST, "Links can only be made for documents and projects".to_string())),
        }
    }

    fn table(self) -> &'static str {
        match self {
            Self::Document => "documents",
            Self::Project => "projects",
        }
    }

    fn name_column(self) -> &'static str {
        match self {
            Self::Document => "title",
            Self::Project => "name",
        }
    }

    fn resource(self, item_id: &str) -> Resource {
        match self {
            Self::Document => Resource::Document(item_id.to_string()),
            Self::Project => Resource::Project(item_id.to_string()),
        }
    }
}

struct LinkRow {
    name: String,
    owner_id: String,
    public_role: Option<String>,
    password_hash: Option<String>,
    expires_at: Option<String>,
}

#[derive(Serialize)]
pub struct LinkStatus {
    pub name: String,
    pub public_role: Option<String>,
    pub requires_password: bool,
    pub expires_at: Option<String>,
    pub expired: bool,
    pub unlocked: bool,
}

#[derive(Deserialize)]
pub struct UnlockRequest {
    pub password: String,
}

#[derive(Deserialize)]
pub struct LinkSettingsRequest {
    pub public_role: Option<String>,
    pub password: Option<String>,
    pub remove_password: Option<bool>,
    pub expires_at: Option<String>,
}

fn grant_token(item_id: &str, password_hash: &str) -> String {
    let fingerprint = crate::hashing::sha256_hex(password_hash.as_bytes());
    format!("{item_id}.{}", &fingerprint[..16])
}

fn is_expired(expires_at: Option<&str>) -> bool {
    expires_at.is_some_and(|expiry| !expiry.is_empty() && expiry <= timestamp_now().as_str())
}

fn is_granted(item_id: &str, password_hash: &str) -> bool {
    let token = grant_token(item_id, password_hash);
    LINK_GRANTS.try_with(|grants| grants.contains(&token)).unwrap_or(false)
}

pub fn public_access(item_id: &str, public_role: Option<&str>, password_hash: Option<&str>, expires_at: Option<&str>) -> Option<AccessRole> {
    let role = public_role.and_then(AccessRole::from_share_role)?;
    if is_expired(expires_at) {
        return None;
    }
    match password_hash.filter(|hash| !hash.is_empty()) {
        Some(hash) if !is_granted(item_id, hash) => None,
        _ => Some(role),
    }
}

fn grants_from_headers(key: &Key, headers: &HeaderMap) -> Vec<String> {
    SignedCookieJar::from_headers(headers, key.clone())
        .get(GRANTS_COOKIE)
        .map(|cookie| cookie.value().split(',').filter(|grant| !grant.is_empty()).map(str::to_string).collect())
        .unwrap_or_default()
}

pub async fn with_grants<F: std::future::Future>(key: &Key, headers: &HeaderMap, work: F) -> F::Output {
    LINK_GRANTS.scope(grants_from_headers(key, headers), work).await
}

async fn load_link(state: &AppState, kind: LinkKind, item_id: &str) -> Result<LinkRow, ApiError> {
    let statement = format!(
        "SELECT {}, owner_id, public_role, link_password_hash, link_expires_at FROM {} WHERE id = $1",
        kind.name_column(),
        kind.table()
    );
    let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>)>(sql_owned(statement))
        .bind(item_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "This link does not exist".to_string()))?;

    let (name, owner_id, public_role, password_hash, expires_at) = row;
    Ok(LinkRow { name, owner_id, public_role, password_hash, expires_at })
}

pub async fn link_status(
    State(state): State<AppState>,
    Path((kind, item_id)): Path<(String, String)>,
) -> Result<Json<LinkStatus>, ApiError> {
    let kind = LinkKind::parse(&kind)?;
    let link = load_link(&state, kind, &item_id).await?;
    if link.public_role.is_none() {
        return Err((StatusCode::NOT_FOUND, "This item is not shared by link".to_string()));
    }

    let password_hash = link.password_hash.clone().filter(|hash| !hash.is_empty());
    Ok(Json(LinkStatus {
        name: link.name,
        public_role: link.public_role,
        requires_password: password_hash.is_some(),
        expired: is_expired(link.expires_at.as_deref()),
        unlocked: password_hash.map(|hash| is_granted(&item_id, &hash)).unwrap_or(true),
        expires_at: link.expires_at,
    }))
}

pub async fn unlock_link(
    State(state): State<AppState>,
    Path((kind, item_id)): Path<(String, String)>,
    client: ClientInfo,
    jar: SignedCookieJar,
    Json(payload): Json<UnlockRequest>,
) -> Result<(SignedCookieJar, StatusCode), ApiError> {
    let kind = LinkKind::parse(&kind)?;
    let link = load_link(&state, kind, &item_id).await?;
    let password_hash = link.password_hash.filter(|hash| !hash.is_empty());
    let (Some(_), Some(password_hash)) = (&link.public_role, password_hash) else {
        return Err((StatusCode::BAD_REQUEST, "This link does not need a password".to_string()));
    };
    if is_expired(link.expires_at.as_deref()) {
        return Err((StatusCode::GONE, "This link has expired".to_string()));
    }

    state.login_guard.ensure_allowed(UNLOCK_SCOPE, &item_id, &client.ip_address)?;
    if !passwords::verify_password(&payload.password, &password_hash) {
        let rules = crate::accounts::auth_settings::load(&state).await.lockout_rules();
        state.login_guard.record_failure(UNLOCK_SCOPE, &item_id, &client.ip_address, rules);
        return Err((StatusCode::UNAUTHORIZED, "Incorrect password".to_string()));
    }

    let token = grant_token(&item_id, &password_hash);
    let mut grants: Vec<String> = jar
        .get(GRANTS_COOKIE)
        .map(|cookie| cookie.value().split(',').map(str::to_string).collect())
        .unwrap_or_default();
    grants.retain(|grant| !grant.is_empty() && !grant.starts_with(&format!("{item_id}.")));
    grants.push(token);
    let overflow = grants.len().saturating_sub(MAX_GRANTS);
    grants.drain(..overflow);

    let mut cookie = Cookie::new(GRANTS_COOKIE, grants.join(","));
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    cookie.set_secure(crate::site_url::uses_https());
    Ok((jar.add(cookie), StatusCode::NO_CONTENT))
}

fn parse_expiry(value: &str) -> Result<Option<String>, ApiError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    let parsed = chrono::DateTime::parse_from_rfc3339(value)
        .map(|time| time.with_timezone(&chrono::Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").map(|time| time.and_utc())
        })
        .map_err(|_| (StatusCode::BAD_REQUEST, "Expiry must be a date and time".to_string()))?;
    Ok(Some(crate::accounts::sessions::format_time(parsed)))
}

pub async fn update_link(
    State(state): State<AppState>,
    Path((kind, item_id)): Path<(String, String)>,
    jar: SignedCookieJar,
    Json(payload): Json<LinkSettingsRequest>,
) -> Result<Json<LinkStatus>, ApiError> {
    let user_id = jar
        .get("session_user_id")
        .map(|cookie| cookie.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;
    let kind = LinkKind::parse(&kind)?;
    let link = load_link(&state, kind, &item_id).await?;
    if link.owner_id != user_id {
        return Err((StatusCode::FORBIDDEN, "Only the owner can change link sharing".to_string()));
    }

    let public_role = match payload.public_role.as_deref().map(str::trim) {
        Some("viewer") => Some("viewer".to_string()),
        Some("editor") => Some("editor".to_string()),
        Some("suggester") if matches!(kind, LinkKind::Document) => Some("suggester".to_string()),
        Some("none") | Some("") => None,
        Some(_) => return Err((StatusCode::BAD_REQUEST, invalid_role_message(kind).to_string())),
        None => link.public_role.clone(),
    };

    let password_hash = match (&payload.password, payload.remove_password.unwrap_or(false)) {
        (_, true) => None,
        (Some(password), false) if !password.is_empty() => {
            Some(passwords::hash_password(password).map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))?)
        }
        _ => link.password_hash.clone(),
    };

    let expires_at = match &payload.expires_at {
        Some(value) => parse_expiry(value)?,
        None => link.expires_at.clone(),
    };

    let statement = format!(
        "UPDATE {} SET public_role = $1, link_password_hash = $2, link_expires_at = $3 WHERE id = $4",
        kind.table()
    );
    sqlx::query(sql_owned(statement))
        .bind(&public_role)
        .bind(&password_hash)
        .bind(&expires_at)
        .bind(&item_id)
        .execute(&state.db)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    access::revalidate_resource(&state, kind.resource(&item_id)).await;

    Ok(Json(LinkStatus {
        name: link.name,
        requires_password: password_hash.is_some(),
        expired: is_expired(expires_at.as_deref()),
        unlocked: true,
        public_role,
        expires_at,
    }))
}

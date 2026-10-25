use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use super::access::editable_project;
use super::accounts::{self, attach_saved_account};
use super::file_rules::FileRules;
use super::import::{import_repository, preview_import, ImportPreview};
use super::local_files;
use super::locks::lock_project;
use super::policy::load_policy;
use super::rate_limit::check_rate;
use super::remote::{self, Provider, RemoteRepo};
use super::repo_settings::{clean_branch, clean_excluded_paths, clean_folder, clean_token, clean_username, RepoUrl};
use super::{internal_error, snapshot, ApiError};
use crate::accounts::signed_in::SignedInUser;
use crate::db::sql;
use crate::sealed_secrets;
use crate::AppState;

type LinkRow = (
    String,
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

fn link_token_name(project_id: &str) -> String {
    format!("git_link:{project_id}")
}

pub struct GitLink {
    pub project_id: String,
    pub provider: Provider,
    pub url: RepoUrl,
    pub branch: String,
    pub folder: String,
    pub username: Option<String>,
    pub token: Option<String>,
    pub account_id: Option<String>,
    pub account_username: Option<String>,
    pub excluded_paths: Vec<String>,
    pub base_commit: Option<String>,
    pub last_synced_at: Option<String>,
    pub created_by_name: Option<String>,
}

#[derive(Serialize)]
pub struct LinkView {
    pub provider: &'static str,
    pub repo_url: String,
    pub branch: String,
    pub folder: String,
    pub username: Option<String>,
    pub has_token: bool,
    pub account_username: Option<String>,
    pub excluded_paths: Vec<String>,
    pub base_commit: Option<String>,
    pub last_synced_at: Option<String>,
    pub created_by_name: Option<String>,
    pub can_manage: bool,
}

#[derive(Serialize)]
pub struct LinkStatus {
    pub enabled: bool,
    pub generic_hosts_enabled: bool,
    pub link: Option<LinkView>,
}

#[derive(Deserialize)]
pub struct LinkRequest {
    pub provider: Option<String>,
    pub repo_url: String,
    pub branch: String,
    pub folder: Option<String>,
    pub username: Option<String>,
    pub token: Option<String>,
    pub excluded_paths: Option<Vec<String>>,
    pub replace_project_files: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateLinkRequest {
    pub token: Option<String>,
    pub use_account: Option<bool>,
    pub username: Option<String>,
    pub excluded_paths: Option<Vec<String>>,
}

impl GitLink {
    pub fn rules(&self) -> FileRules {
        FileRules::new(self.excluded_paths.clone())
    }

    pub fn remote(&self) -> RemoteRepo {
        RemoteRepo {
            provider: self.provider,
            url: self.url.clone(),
            branch: self.branch.clone(),
            folder: self.folder.clone(),
            username: self.username.clone(),
            token: self.token.clone(),
            cache_name: self.project_id.clone(),
        }
    }

    pub fn view(&self, can_manage: bool) -> LinkView {
        LinkView {
            provider: self.provider.as_str(),
            repo_url: self.url.text.clone(),
            branch: self.branch.clone(),
            folder: self.folder.clone(),
            username: self.username.clone(),
            has_token: self.token.is_some(),
            account_username: self.account_username.clone(),
            excluded_paths: self.excluded_paths.clone(),
            base_commit: self.base_commit.clone(),
            last_synced_at: self.last_synced_at.clone(),
            created_by_name: self.created_by_name.clone(),
            can_manage,
        }
    }
}

fn split_paths(stored: Option<String>) -> Vec<String> {
    stored.unwrap_or_default().lines().map(str::to_string).filter(|line| !line.is_empty()).collect()
}

fn link_from_row(state: &AppState, row: LinkRow) -> Result<GitLink, ApiError> {
    let (
        project_id,
        provider,
        repo_url,
        branch,
        folder,
        username,
        token_sealed,
        excluded_paths,
        base_commit,
        last_synced_at,
        created_by_name,
        account_id,
        account_owner,
        account_username,
        account_token_sealed,
    ) = row;
    let broken = || (StatusCode::INTERNAL_SERVER_ERROR, "The saved Git link is damaged. Unlink and link the repository again.".to_string());
    let token = match (&account_id, account_owner, account_token_sealed) {
        (Some(account_id), Some(owner), Some(sealed)) => sealed_secrets::open(&state.key, &accounts::token_name(&owner, account_id), &sealed),
        _ => token_sealed.and_then(|sealed| sealed_secrets::open(&state.key, &link_token_name(&project_id), &sealed)),
    };
    Ok(GitLink {
        provider: Provider::parse(&provider).ok_or_else(broken)?,
        url: RepoUrl::parse(&repo_url).map_err(|_| broken())?,
        token,
        account_username: account_id.as_ref().and(account_username),
        account_id,
        excluded_paths: split_paths(excluded_paths),
        project_id,
        branch,
        folder,
        username,
        base_commit,
        last_synced_at,
        created_by_name,
    })
}

pub async fn load_link(state: &AppState, project_id: &str) -> Result<Option<GitLink>, ApiError> {
    let row = sqlx::query_as::<_, LinkRow>(sql(
        "SELECT l.project_id, l.provider, l.repo_url, l.branch, l.folder, l.username, l.token_sealed, l.excluded_paths, l.base_commit, l.last_synced_at, u.username, \
         a.id, a.user_id, a.username, a.token_sealed \
         FROM git_links l LEFT JOIN users u ON u.id = l.created_by LEFT JOIN git_accounts a ON a.id = l.account_id WHERE l.project_id = $1",
    ))
    .bind(project_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?;
    row.map(|row| link_from_row(state, row)).transpose()
}

pub async fn require_link(state: &AppState, project_id: &str) -> Result<GitLink, ApiError> {
    load_link(state, project_id)
        .await?
        .ok_or((StatusCode::NOT_FOUND, "This project is not linked to a Git repository".to_string()))
}

fn link_from_request(project_id: &str, payload: &LinkRequest) -> Result<GitLink, ApiError> {
    let url = RepoUrl::parse(&payload.repo_url)?;
    let provider = match payload.provider.as_deref().filter(|provider| !provider.is_empty() && *provider != "auto") {
        Some(provider) => Provider::parse(provider).ok_or((StatusCode::BAD_REQUEST, "Unknown Git host type".to_string()))?,
        None => Provider::detect(&url),
    };
    Ok(GitLink {
        project_id: project_id.to_string(),
        provider,
        url,
        branch: clean_branch(&payload.branch)?,
        folder: clean_folder(payload.folder.as_deref().unwrap_or_default())?,
        username: clean_username(payload.username.as_deref()),
        token: clean_token(payload.token.as_deref()),
        account_id: None,
        account_username: None,
        excluded_paths: clean_excluded_paths(payload.excluded_paths.as_deref().unwrap_or_default())?,
        base_commit: None,
        last_synced_at: None,
        created_by_name: None,
    })
}

pub(super) async fn link_for_user(state: &AppState, project_id: &str, payload: &LinkRequest, user_id: &str) -> Result<GitLink, ApiError> {
    let mut link = link_from_request(project_id, payload)?;
    attach_saved_account(state, user_id, &mut link).await?;
    Ok(link)
}

fn sealed_link_token(state: &AppState, link: &GitLink) -> Option<String> {
    match (&link.account_id, &link.token) {
        (None, Some(token)) => Some(sealed_secrets::seal(&state.key, &link_token_name(&link.project_id), token)),
        _ => None,
    }
}

pub(super) async fn save_new_link(state: &AppState, link: &GitLink, user_id: &str) -> Result<(), ApiError> {
    sqlx::query(sql(
        "INSERT INTO git_links (project_id, created_by, provider, repo_url, branch, folder, username, token_sealed, account_id, excluded_paths) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    ))
    .bind(&link.project_id)
    .bind(user_id)
    .bind(link.provider.as_str())
    .bind(&link.url.text)
    .bind(&link.branch)
    .bind(&link.folder)
    .bind(&link.username)
    .bind(sealed_link_token(state, link))
    .bind(&link.account_id)
    .bind(link.excluded_paths.join("\n"))
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::CONFLICT, "This project is already linked to a Git repository".to_string()))?;
    Ok(())
}

pub(super) async fn delete_link_rows(state: &AppState, project_id: &str) -> Result<(), ApiError> {
    snapshot::clear(state, project_id).await?;
    sqlx::query(sql("DELETE FROM git_links WHERE project_id = $1"))
        .bind(project_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(())
}

pub async fn link_status(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<Json<LinkStatus>, ApiError> {
    let access = editable_project(&state, &project_id, &user_id).await?;
    let policy = load_policy(&state).await;
    let link = load_link(&state, &project_id).await?;
    Ok(Json(LinkStatus {
        enabled: policy.enabled,
        generic_hosts_enabled: policy.generic_hosts_enabled,
        link: link.map(|link| link.view(access.is_owner)),
    }))
}

pub async fn preview_link(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<LinkRequest>,
) -> Result<Json<ImportPreview>, ApiError> {
    let access = editable_project(&state, &project_id, &user_id).await?;
    access.require_owner()?;
    let link = link_for_user(&state, &project_id, &payload, &user_id).await?;
    load_policy(&state).await.check_provider(link.provider)?;
    check_rate(&user_id)?;
    let _guard = lock_project(&project_id)?;
    let local = local_files::load(&state, &project_id, &link.rules()).await?;
    let (preview, _) = preview_import(&state, &access.project.owner_id, &local, &link, payload.replace_project_files.unwrap_or(true)).await?;
    Ok(Json(preview))
}

pub async fn create_link(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<LinkRequest>,
) -> Result<Json<LinkStatus>, ApiError> {
    let access = editable_project(&state, &project_id, &user_id).await?;
    access.require_owner()?;
    let link = link_for_user(&state, &project_id, &payload, &user_id).await?;
    let policy = load_policy(&state).await;
    policy.check_provider(link.provider)?;
    check_rate(&user_id)?;
    let _guard = lock_project(&project_id)?;
    if load_link(&state, &project_id).await?.is_some() {
        return Err((StatusCode::CONFLICT, "This project is already linked to a Git repository".to_string()));
    }

    let replace_project_files = payload.replace_project_files.unwrap_or(true);
    let local = local_files::load(&state, &project_id, &link.rules()).await?;
    let (preview, entries) = preview_import(&state, &access.project.owner_id, &local, &link, replace_project_files).await?;
    save_new_link(&state, &link, &user_id).await?;
    if let Err(error) = import_repository(&state, &access.project, &link, &preview.commit, entries, replace_project_files, &user_id).await {
        let _ = delete_link_rows(&state, &project_id).await;
        return Err(error);
    }

    let link = require_link(&state, &project_id).await?;
    Ok(Json(LinkStatus { enabled: policy.enabled, generic_hosts_enabled: policy.generic_hosts_enabled, link: Some(link.view(true)) }))
}

pub async fn update_link(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<UpdateLinkRequest>,
) -> Result<Json<LinkView>, ApiError> {
    let access = editable_project(&state, &project_id, &user_id).await?;
    access.require_owner()?;
    let _guard = lock_project(&project_id)?;
    let mut link = require_link(&state, &project_id).await?;

    if let Some(token) = &payload.token {
        link.token = clean_token(Some(token));
        link.account_id = None;
        link.account_username = None;
    }
    if payload.use_account == Some(true) {
        link.token = None;
        if !attach_saved_account(&state, &user_id, &mut link).await? {
            return Err((StatusCode::BAD_REQUEST, format!("You have no saved account for {}. Add one in Settings, Git.", link.url.web_base())));
        }
    }
    if let Some(username) = &payload.username {
        link.username = clean_username(Some(username));
    }
    if let Some(excluded_paths) = &payload.excluded_paths {
        link.excluded_paths = clean_excluded_paths(excluded_paths)?;
    }

    sqlx::query(sql("UPDATE git_links SET username = $1, token_sealed = $2, account_id = $3, excluded_paths = $4 WHERE project_id = $5"))
        .bind(&link.username)
        .bind(sealed_link_token(&state, &link))
        .bind(&link.account_id)
        .bind(link.excluded_paths.join("\n"))
        .bind(&project_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(Json(link.view(true)))
}

pub async fn delete_link(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<StatusCode, ApiError> {
    let access = editable_project(&state, &project_id, &user_id).await?;
    access.require_owner()?;
    let _guard = lock_project(&project_id)?;
    delete_link_rows(&state, &project_id).await?;
    remote::forget_cache(&project_id);
    Ok(StatusCode::NO_CONTENT)
}

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::links::GitLink;
use super::policy::load_policy;
use super::rate_limit::check_rate;
use super::remote::{self, Provider};
use super::repo_settings::{clean_token, RepoUrl};
use super::{internal_error, ApiError};
use crate::accounts::signed_in::SignedInUser;
use crate::db::sql;
use crate::sealed_secrets;
use crate::AppState;

const MAX_ACCOUNTS: i64 = 20;
const MAX_TOKEN_LENGTH: usize = 1024;
const HINT_LENGTH: usize = 4;
const MIN_HINTED_TOKEN_LENGTH: usize = 12;

type AccountRow = (String, String, String, String, Option<String>, String);

#[derive(Serialize)]
pub struct AccountView {
    pub id: String,
    pub provider: &'static str,
    pub base_url: String,
    pub username: String,
    pub token_hint: Option<String>,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct ConnectRequest {
    pub provider: String,
    pub server: Option<String>,
    pub token: String,
}

pub struct SavedAccount {
    pub id: String,
    pub username: String,
    pub token: String,
}

pub fn token_name(user_id: &str, account_id: &str) -> String {
    format!("git_account:{user_id}:{account_id}")
}

fn account_provider(value: &str) -> Result<Provider, ApiError> {
    match Provider::parse(value) {
        Some(provider @ (Provider::Github | Provider::Gitlab)) => Ok(provider),
        _ => Err((StatusCode::BAD_REQUEST, "Saved accounts are only for GitHub and GitLab".to_string())),
    }
}

fn default_server(provider: Provider) -> &'static str {
    match provider {
        Provider::Gitlab => "gitlab.com",
        _ => "github.com",
    }
}

fn token_hint(token: &str) -> Option<String> {
    let characters: Vec<char> = token.chars().collect();
    (characters.len() >= MIN_HINTED_TOKEN_LENGTH).then(|| characters[characters.len() - HINT_LENGTH..].iter().collect())
}

fn view_from_row(row: AccountRow) -> Option<AccountView> {
    let (id, provider, base_url, username, token_hint, created_at) = row;
    Some(AccountView { id, provider: Provider::parse(&provider)?.as_str(), base_url, username, token_hint, created_at })
}

async fn list_for(state: &AppState, user_id: &str) -> Result<Vec<AccountView>, ApiError> {
    let rows = sqlx::query_as::<_, AccountRow>(sql(
        "SELECT id, provider, base_url, username, token_hint, created_at FROM git_accounts WHERE user_id = $1 ORDER BY provider ASC, base_url ASC",
    ))
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(rows.into_iter().filter_map(view_from_row).collect())
}

pub async fn saved_account(state: &AppState, user_id: &str, provider: Provider, server: &RepoUrl) -> Result<Option<SavedAccount>, ApiError> {
    let row = sqlx::query_as::<_, (String, String, String)>(sql(
        "SELECT id, username, token_sealed FROM git_accounts WHERE user_id = $1 AND provider = $2 AND base_url = $3",
    ))
    .bind(user_id)
    .bind(provider.as_str())
    .bind(server.web_base())
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(row.and_then(|(id, username, sealed)| {
        let token = sealed_secrets::open(&state.key, &token_name(user_id, &id), &sealed)?;
        Some(SavedAccount { id, username, token })
    }))
}

pub async fn attach_saved_account(state: &AppState, user_id: &str, link: &mut GitLink) -> Result<bool, ApiError> {
    if link.token.is_some() || link.provider == Provider::Generic {
        return Ok(false);
    }
    let Some(account) = saved_account(state, user_id, link.provider, &link.url).await? else {
        return Ok(false);
    };
    link.token = Some(account.token);
    link.account_id = Some(account.id);
    link.account_username = Some(account.username);
    Ok(true)
}

pub async fn list_accounts(State(state): State<AppState>, SignedInUser(user_id): SignedInUser) -> Result<Json<Vec<AccountView>>, ApiError> {
    Ok(Json(list_for(&state, &user_id).await?))
}

pub async fn connect_account(
    State(state): State<AppState>,
    SignedInUser(user_id): SignedInUser,
    Json(payload): Json<ConnectRequest>,
) -> Result<Json<Vec<AccountView>>, ApiError> {
    load_policy(&state).await.check_enabled()?;
    let provider = account_provider(&payload.provider)?;
    let server_input = payload.server.as_deref().map(str::trim).filter(|server| !server.is_empty()).unwrap_or(default_server(provider));
    let server = RepoUrl::parse_server(server_input)?;
    let token = clean_token(Some(&payload.token)).ok_or((StatusCode::BAD_REQUEST, "Paste an access token".to_string()))?;
    if token.len() > MAX_TOKEN_LENGTH || token.chars().any(|character| character.is_control() || character.is_whitespace()) {
        return Err((StatusCode::BAD_REQUEST, "That does not look like an access token".to_string()));
    }
    check_rate(&user_id)?;
    let username = remote::token_owner(provider, &server, &token).await?;

    let existing = sqlx::query_as::<_, (String,)>(sql("SELECT id FROM git_accounts WHERE user_id = $1 AND provider = $2 AND base_url = $3"))
        .bind(&user_id)
        .bind(provider.as_str())
        .bind(server.web_base())
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;

    match existing {
        Some((account_id,)) => {
            sqlx::query(sql("UPDATE git_accounts SET username = $1, token_sealed = $2, token_hint = $3 WHERE id = $4 AND user_id = $5"))
                .bind(&username)
                .bind(sealed_secrets::seal(&state.key, &token_name(&user_id, &account_id), &token))
                .bind(token_hint(&token))
                .bind(&account_id)
                .bind(&user_id)
                .execute(&state.db)
                .await
                .map_err(internal_error)?;
        }
        None => {
            let (count,) = sqlx::query_as::<_, (i64,)>(sql("SELECT COUNT(*) FROM git_accounts WHERE user_id = $1"))
                .bind(&user_id)
                .fetch_one(&state.db)
                .await
                .map_err(internal_error)?;
            if count >= MAX_ACCOUNTS {
                return Err((StatusCode::BAD_REQUEST, format!("You can save up to {MAX_ACCOUNTS} Git accounts")));
            }
            let account_id = Uuid::new_v4().to_string();
            sqlx::query(sql(
                "INSERT INTO git_accounts (id, user_id, provider, base_url, username, token_sealed, token_hint) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            ))
            .bind(&account_id)
            .bind(&user_id)
            .bind(provider.as_str())
            .bind(server.web_base())
            .bind(&username)
            .bind(sealed_secrets::seal(&state.key, &token_name(&user_id, &account_id), &token))
            .bind(token_hint(&token))
            .execute(&state.db)
            .await
            .map_err(internal_error)?;
        }
    }
    Ok(Json(list_for(&state, &user_id).await?))
}

pub async fn delete_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<StatusCode, ApiError> {
    let mut transaction = state.db.begin().await.map_err(internal_error)?;
    let deleted = sqlx::query(sql("DELETE FROM git_accounts WHERE id = $1 AND user_id = $2"))
        .bind(&account_id)
        .bind(&user_id)
        .execute(&mut *transaction)
        .await
        .map_err(internal_error)?;
    if deleted.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Account not found".to_string()));
    }
    sqlx::query(sql("UPDATE git_links SET account_id = NULL WHERE account_id = $1"))
        .bind(&account_id)
        .execute(&mut *transaction)
        .await
        .map_err(internal_error)?;
    transaction.commit().await.map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hints_only_long_tokens() {
        assert_eq!(token_hint("ghp_abcdefghijklmnop1234").as_deref(), Some("1234"));
        assert_eq!(token_hint("short"), None);
    }

    #[test]
    fn binds_sealed_tokens_to_their_owner() {
        let key = axum_extra::extract::cookie::Key::generate();
        let sealed = sealed_secrets::seal(&key, &token_name("alice", "one"), "secret-token");
        assert_eq!(sealed_secrets::open(&key, &token_name("alice", "one"), &sealed).as_deref(), Some("secret-token"));
        assert_eq!(sealed_secrets::open(&key, &token_name("mallory", "one"), &sealed), None);
        assert_eq!(sealed_secrets::open(&key, &token_name("alice", "two"), &sealed), None);
        assert!(!sealed.contains("secret-token"));
    }
}

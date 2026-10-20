use uuid::Uuid;

use super::{ExternalProfile, ProviderConfig};
use crate::accounts::{auth_settings::AuthSettings, lookup, passwords};
use crate::db::sql;
use crate::AppState;

const MAX_USERNAME_ATTEMPTS: u32 = 50;

async fn identity_owner(state: &AppState, provider_id: &str, subject: &str) -> Result<Option<String>, String> {
    sqlx::query_as::<_, (String,)>(sql("SELECT user_id FROM user_identities WHERE provider = $1 AND subject = $2"))
        .bind(provider_id)
        .bind(subject)
        .fetch_optional(&state.db)
        .await
        .map(|row| row.map(|(user_id,)| user_id))
        .map_err(|error| error.to_string())
}

pub async fn attach_identity(state: &AppState, user_id: &str, provider_id: &str, profile: &ExternalProfile) -> Result<(), String> {
    sqlx::query(sql(
        "INSERT INTO user_identities (id, user_id, provider, subject, email, created_at) VALUES ($1, $2, $3, $4, $5, $6)",
    ))
    .bind(Uuid::new_v4().to_string())
    .bind(user_id)
    .bind(provider_id)
    .bind(&profile.subject)
    .bind(&profile.email)
    .bind(crate::db::timestamp_now())
    .execute(&state.db)
    .await
    .map(|_| ())
    .map_err(|error| error.to_string())
}

fn clean_username(candidate: &str) -> String {
    let cleaned: String = candidate
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
        .take(40)
        .collect();
    if cleaned.is_empty() { "user".to_string() } else { cleaned }
}

async fn username_taken(state: &AppState, username: &str) -> bool {
    sqlx::query_as::<_, (String,)>(sql("SELECT id FROM users WHERE LOWER(username) = LOWER($1)"))
        .bind(username)
        .fetch_optional(&state.db)
        .await
        .map(|row| row.is_some())
        .unwrap_or(true)
}

async fn available_username(state: &AppState, profile: &ExternalProfile, email: &str) -> String {
    let base = profile
        .preferred_username
        .as_deref()
        .map(|name| name.split('@').next().unwrap_or(name))
        .or_else(|| email.split('@').next())
        .map(clean_username)
        .unwrap_or_else(|| "user".to_string());

    for attempt in 1..=MAX_USERNAME_ATTEMPTS {
        let candidate = if attempt == 1 { base.clone() } else { format!("{base}{attempt}") };
        if !username_taken(state, &candidate).await {
            return candidate;
        }
    }
    format!("{base}-{}", &Uuid::new_v4().simple().to_string()[..8])
}

async fn create_account(state: &AppState, provider: &ProviderConfig, profile: &ExternalProfile, email: &str) -> Result<String, String> {
    let policy = crate::accounts::policy::load_policy(state).await;
    policy.check_email_domain(email).map_err(|(_, message)| message)?;

    let user_id = Uuid::new_v4().to_string();
    let username = available_username(state, profile, email).await;
    sqlx::query(sql(
        "INSERT INTO users (id, username, email, password_hash, email_verified) VALUES ($1, $2, $3, $4, $5)",
    ))
    .bind(&user_id)
    .bind(&username)
    .bind(email)
    .bind(passwords::NO_PASSWORD)
    .bind(1i64)
    .execute(&state.db)
    .await
    .map_err(|error| error.to_string())?;

    attach_identity(state, &user_id, &provider.id, profile).await?;
    tracing::info!("Created account {username} from {} sign-in", provider.name);
    Ok(user_id)
}

pub async fn resolve_account(
    state: &AppState,
    provider: &ProviderConfig,
    profile: &ExternalProfile,
    settings: &AuthSettings,
) -> Result<String, String> {
    if let Some(user_id) = identity_owner(state, &provider.id, &profile.subject).await? {
        return Ok(user_id);
    }

    let verified_email = profile.email.as_deref().filter(|_| profile.email_verified);
    let Some(email) = verified_email else {
        return Err(format!("{} did not share a verified email address, so the account could not be matched", provider.name));
    };

    if let Some(user) = lookup::user_by_email(state, email).await.map_err(|(_, message)| message)? {
        attach_identity(state, &user.id, &provider.id, profile).await?;
        let _ = sqlx::query(sql("UPDATE users SET email_verified = 1 WHERE id = $1"))
            .bind(&user.id)
            .execute(&state.db)
            .await;
        return Ok(user.id);
    }

    if !settings.auto_create_accounts {
        return Err("No account uses this email address. Ask an administrator to create one for you.".to_string());
    }
    create_account(state, provider, profile, email).await
}

pub async fn link_to_user(state: &AppState, user_id: &str, provider: &ProviderConfig, profile: &ExternalProfile) -> Result<(), String> {
    match identity_owner(state, &provider.id, &profile.subject).await? {
        Some(owner) if owner == user_id => Ok(()),
        Some(_) => Err(format!("This {} account is already connected to another user", provider.name)),
        None => attach_identity(state, user_id, &provider.id, profile).await,
    }
}

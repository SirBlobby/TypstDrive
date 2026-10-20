use serde_json::Value;

use super::http::{get_json, post_form, query_string};
use super::{ExternalProfile, ProviderConfig};

const AUTHORIZE_URL: &str = "https://github.com/login/oauth/authorize";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const USER_URL: &str = "https://api.github.com/user";
const EMAILS_URL: &str = "https://api.github.com/user/emails";

pub fn authorization_url(provider: &ProviderConfig, redirect_uri: &str, state: &str) -> String {
    let pairs = [
        ("client_id", provider.client_id.as_str()),
        ("redirect_uri", redirect_uri),
        ("scope", provider.scopes.as_str()),
        ("state", state),
        ("allow_signup", "false"),
    ];
    format!("{AUTHORIZE_URL}?{}", query_string(&pairs))
}

fn primary_verified_email(emails: &Value) -> Option<String> {
    let entries = emails.as_array()?;
    let verified = |entry: &&Value| entry.get("verified").and_then(Value::as_bool) == Some(true);
    let primary = |entry: &&Value| entry.get("primary").and_then(Value::as_bool) == Some(true);
    entries
        .iter()
        .filter(verified)
        .find(primary)
        .or_else(|| entries.iter().find(verified))
        .and_then(|entry| entry.get("email").and_then(Value::as_str))
        .map(str::to_string)
}

pub async fn exchange_code(provider: &ProviderConfig, redirect_uri: &str, code: &str) -> Result<ExternalProfile, String> {
    let form = vec![
        ("client_id".to_string(), provider.client_id.clone()),
        ("client_secret".to_string(), provider.client_secret.clone()),
        ("code".to_string(), code.to_string()),
        ("redirect_uri".to_string(), redirect_uri.to_string()),
    ];
    let tokens = post_form(TOKEN_URL.to_string(), form, None).await?;
    let access_token = tokens
        .get("access_token")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            let reason = tokens.get("error_description").and_then(Value::as_str).unwrap_or("no access token was returned");
            format!("GitHub sign-in failed: {reason}")
        })?;

    let user = get_json(USER_URL.to_string(), Some(access_token.clone())).await?;
    let subject = user
        .get("id")
        .and_then(Value::as_i64)
        .map(|id| id.to_string())
        .ok_or("GitHub did not return a user id")?;
    let emails = get_json(EMAILS_URL.to_string(), Some(access_token)).await.unwrap_or(Value::Null);
    let email = primary_verified_email(&emails);

    Ok(ExternalProfile {
        subject,
        email_verified: email.is_some(),
        email,
        preferred_username: user.get("login").and_then(Value::as_str).map(str::to_string),
        display_name: user.get("name").and_then(Value::as_str).map(str::to_string),
    })
}

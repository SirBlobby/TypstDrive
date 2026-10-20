pub mod handlers;
mod github;
mod http;
mod identities;
mod linking;
mod oidc;
mod state_cookie;

use std::collections::HashMap;

use tokio::sync::Mutex;

pub use identities::{list_identities, unlink_identity};

#[derive(Clone)]
pub enum ProviderKind {
    Oidc { issuer: String },
    GitHub,
}

impl ProviderKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Oidc { .. } => "oidc",
            Self::GitHub => "github",
        }
    }
}

#[derive(Clone)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub kind: ProviderKind,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: String,
    pub trust_email: bool,
    pub hosted_domain: Option<String>,
    pub secret_in_body: bool,
}

impl ProviderConfig {
    pub fn callback_url(&self) -> Option<String> {
        crate::site_url::link(&format!("/api/auth/sso/{}/callback", self.id))
    }
}

pub struct SsoRegistry {
    providers: Vec<ProviderConfig>,
    problems: Vec<String>,
    discovered: Mutex<HashMap<String, oidc::OidcEndpoints>>,
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name).ok().map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

fn env_flag(name: &str) -> bool {
    env_value(name).is_some_and(|value| value.eq_ignore_ascii_case("true") || value == "1")
}

fn title_case(id: &str) -> String {
    let mut characters = id.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => String::new(),
    }
}

fn is_valid_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-')
}

fn oidc_provider(id: &str, problems: &mut Vec<String>) -> Option<ProviderConfig> {
    let prefix = format!("OIDC_{}", id.to_uppercase().replace('-', "_"));
    let issuer = env_value(&format!("{prefix}_ISSUER"));
    let client_id = env_value(&format!("{prefix}_CLIENT_ID"));
    let client_secret = env_value(&format!("{prefix}_CLIENT_SECRET"));

    let (Some(issuer), Some(client_id), Some(client_secret)) = (issuer, client_id, client_secret) else {
        problems.push(format!("Provider '{id}' needs {prefix}_ISSUER, {prefix}_CLIENT_ID and {prefix}_CLIENT_SECRET"));
        return None;
    };

    Some(ProviderConfig {
        id: id.to_string(),
        name: env_value(&format!("{prefix}_NAME")).unwrap_or_else(|| title_case(id)),
        kind: ProviderKind::Oidc { issuer: issuer.trim_end_matches('/').to_string() },
        client_id,
        client_secret,
        scopes: env_value(&format!("{prefix}_SCOPES")).unwrap_or_else(|| "openid email profile".to_string()),
        trust_email: env_flag(&format!("{prefix}_TRUST_EMAIL")),
        hosted_domain: None,
        secret_in_body: env_value(&format!("{prefix}_TOKEN_AUTH")).is_some_and(|value| value.eq_ignore_ascii_case("post")),
    })
}

fn google_provider() -> Option<ProviderConfig> {
    Some(ProviderConfig {
        id: "google".to_string(),
        name: "Google".to_string(),
        kind: ProviderKind::Oidc { issuer: "https://accounts.google.com".to_string() },
        client_id: env_value("GOOGLE_CLIENT_ID")?,
        client_secret: env_value("GOOGLE_CLIENT_SECRET")?,
        scopes: "openid email profile".to_string(),
        trust_email: false,
        hosted_domain: env_value("GOOGLE_HOSTED_DOMAIN").map(|domain| domain.to_lowercase()),
        secret_in_body: true,
    })
}

fn github_provider() -> Option<ProviderConfig> {
    Some(ProviderConfig {
        id: "github".to_string(),
        name: "GitHub".to_string(),
        kind: ProviderKind::GitHub,
        client_id: env_value("GITHUB_CLIENT_ID")?,
        client_secret: env_value("GITHUB_CLIENT_SECRET")?,
        scopes: "read:user user:email".to_string(),
        trust_email: false,
        hosted_domain: None,
        secret_in_body: true,
    })
}

impl SsoRegistry {
    pub fn from_environment() -> Self {
        let mut providers = Vec::new();
        let mut problems = Vec::new();

        let configured_ids = env_value("OIDC_PROVIDERS").unwrap_or_default();
        for id in configured_ids.split(',').map(|id| id.trim().to_lowercase()).filter(|id| !id.is_empty()) {
            if !is_valid_id(&id) || id == "google" || id == "github" {
                problems.push(format!("'{id}' is not a valid provider name. Use lowercase letters, digits and dashes."));
                continue;
            }
            if let Some(provider) = oidc_provider(&id, &mut problems) {
                providers.push(provider);
            }
        }
        providers.extend(google_provider());
        providers.extend(github_provider());

        if !providers.is_empty() && crate::site_url::public_url().is_none() {
            problems.push("PUBLIC_URL must be set before single sign-on can be used".to_string());
        }
        for problem in &problems {
            tracing::warn!("Single sign-on: {problem}");
        }

        Self { providers, problems, discovered: Mutex::new(HashMap::new()) }
    }

    pub fn configured_providers(&self) -> impl Iterator<Item = &ProviderConfig> {
        self.providers.iter()
    }

    pub fn usable_providers(&self) -> impl Iterator<Item = &ProviderConfig> {
        let usable = crate::site_url::public_url().is_some();
        self.providers.iter().filter(move |_| usable)
    }

    pub fn find(&self, provider_id: &str) -> Option<&ProviderConfig> {
        self.usable_providers().find(|provider| provider.id == provider_id)
    }

    pub fn problems(&self) -> &[String] {
        &self.problems
    }

    async fn endpoints(&self, issuer: &str) -> Result<oidc::OidcEndpoints, String> {
        if let Some(endpoints) = self.discovered.lock().await.get(issuer) {
            return Ok(endpoints.clone());
        }
        let endpoints = oidc::discover(issuer).await?;
        self.discovered.lock().await.insert(issuer.to_string(), endpoints.clone());
        Ok(endpoints)
    }
}

pub struct ExternalProfile {
    pub subject: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub preferred_username: Option<String>,
    pub display_name: Option<String>,
}

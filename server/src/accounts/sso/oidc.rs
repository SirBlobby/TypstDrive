use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde_json::Value;

use super::http::{basic_credentials, get_json, post_form, query_string};
use super::{ExternalProfile, ProviderConfig};

#[derive(Clone)]
pub struct OidcEndpoints {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
}

fn text_field(document: &Value, name: &str) -> Option<String> {
    document.get(name).and_then(Value::as_str).map(str::to_string).filter(|value| !value.is_empty())
}

fn bool_field(document: &Value, name: &str) -> bool {
    match document.get(name) {
        Some(Value::Bool(value)) => *value,
        Some(Value::String(value)) => value.eq_ignore_ascii_case("true"),
        _ => false,
    }
}

pub async fn discover(issuer: &str) -> Result<OidcEndpoints, String> {
    let document = get_json(format!("{issuer}/.well-known/openid-configuration"), None).await?;
    let required = |name: &str| text_field(&document, name).ok_or(format!("the discovery document has no {name}"));

    Ok(OidcEndpoints {
        issuer: text_field(&document, "issuer").unwrap_or_else(|| issuer.to_string()),
        authorization_endpoint: required("authorization_endpoint")?,
        token_endpoint: required("token_endpoint")?,
        userinfo_endpoint: text_field(&document, "userinfo_endpoint"),
    })
}

pub fn authorization_url(
    endpoints: &OidcEndpoints,
    provider: &ProviderConfig,
    redirect_uri: &str,
    state: &str,
    nonce: &str,
    code_challenge: &str,
) -> String {
    let mut pairs = vec![
        ("response_type", "code"),
        ("client_id", provider.client_id.as_str()),
        ("redirect_uri", redirect_uri),
        ("scope", provider.scopes.as_str()),
        ("state", state),
        ("nonce", nonce),
        ("code_challenge", code_challenge),
        ("code_challenge_method", "S256"),
    ];
    if let Some(domain) = &provider.hosted_domain {
        pairs.push(("hd", domain.as_str()));
    }
    let separator = if endpoints.authorization_endpoint.contains('?') { '&' } else { '?' };
    format!("{}{separator}{}", endpoints.authorization_endpoint, query_string(&pairs))
}

fn decode_id_token(id_token: &str) -> Result<Value, String> {
    let payload = id_token.split('.').nth(1).ok_or("the ID token is malformed")?;
    let bytes = URL_SAFE_NO_PAD
        .decode(payload.trim_end_matches('='))
        .map_err(|_| "the ID token is malformed".to_string())?;
    serde_json::from_slice(&bytes).map_err(|_| "the ID token is malformed".to_string())
}

fn audience_matches(claims: &Value, client_id: &str) -> bool {
    match claims.get("aud") {
        Some(Value::String(audience)) => audience == client_id,
        Some(Value::Array(audiences)) => audiences.iter().any(|audience| audience.as_str() == Some(client_id)),
        _ => false,
    }
}

fn check_claims(claims: &Value, endpoints: &OidcEndpoints, provider: &ProviderConfig, nonce: &str) -> Result<(), String> {
    if text_field(claims, "nonce").as_deref() != Some(nonce) {
        return Err("the sign-in response did not match this request".to_string());
    }
    if !audience_matches(claims, &provider.client_id) {
        return Err("the ID token was issued for a different application".to_string());
    }
    let issuer_is_template = endpoints.issuer.contains('{');
    if !issuer_is_template && text_field(claims, "iss").as_deref() != Some(endpoints.issuer.as_str()) {
        return Err("the ID token came from an unexpected issuer".to_string());
    }
    if let Some(expected_domain) = &provider.hosted_domain {
        if text_field(claims, "hd").map(|domain| domain.to_lowercase()).as_deref() != Some(expected_domain.as_str()) {
            return Err(format!("only {expected_domain} accounts can sign in"));
        }
    }
    Ok(())
}

pub async fn exchange_code(
    endpoints: &OidcEndpoints,
    provider: &ProviderConfig,
    redirect_uri: &str,
    code: &str,
    code_verifier: &str,
    nonce: &str,
) -> Result<ExternalProfile, String> {
    let mut form = vec![
        ("grant_type".to_string(), "authorization_code".to_string()),
        ("code".to_string(), code.to_string()),
        ("redirect_uri".to_string(), redirect_uri.to_string()),
        ("code_verifier".to_string(), code_verifier.to_string()),
        ("client_id".to_string(), provider.client_id.clone()),
    ];
    let authorization = if provider.secret_in_body {
        form.push(("client_secret".to_string(), provider.client_secret.clone()));
        None
    } else {
        Some(basic_credentials(&provider.client_id, &provider.client_secret))
    };

    let tokens = post_form(endpoints.token_endpoint.clone(), form, authorization).await?;
    let id_token = text_field(&tokens, "id_token").ok_or("the provider did not return an ID token")?;
    let claims = decode_id_token(&id_token)?;
    check_claims(&claims, endpoints, provider, nonce)?;

    let mut profile = ExternalProfile {
        subject: text_field(&claims, "sub").ok_or("the ID token has no subject")?,
        email: text_field(&claims, "email"),
        email_verified: bool_field(&claims, "email_verified"),
        preferred_username: text_field(&claims, "preferred_username"),
        display_name: text_field(&claims, "name"),
    };

    let access_token = text_field(&tokens, "access_token");
    if let (None, Some(userinfo_endpoint), Some(access_token)) = (&profile.email, &endpoints.userinfo_endpoint, access_token) {
        let userinfo = get_json(userinfo_endpoint.clone(), Some(access_token)).await?;
        if text_field(&userinfo, "sub").as_deref() == Some(profile.subject.as_str()) {
            profile.email = text_field(&userinfo, "email");
            profile.email_verified = bool_field(&userinfo, "email_verified");
            profile.preferred_username = profile.preferred_username.or_else(|| text_field(&userinfo, "preferred_username"));
            profile.display_name = profile.display_name.or_else(|| text_field(&userinfo, "name"));
        }
    }

    if provider.trust_email && profile.email.is_some() {
        profile.email_verified = true;
    }
    Ok(profile)
}

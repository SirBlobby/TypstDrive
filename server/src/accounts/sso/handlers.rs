use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::{CookieJar, PrivateCookieJar, SignedCookieJar};
use serde::Deserialize;

use super::http::encode_component;
use super::state_cookie::{self, PendingSignIn};
use super::{github, linking, oidc, ExternalProfile, ProviderConfig, ProviderKind};
use crate::accounts::{auth_settings, client_info::ClientInfo, sessions};
use crate::AppState;

const LOGIN_PAGE: &str = "/login";
const ACCOUNT_PAGE: &str = "/settings?section=security";

#[derive(Deserialize)]
pub struct StartQuery {
    pub redirect: Option<String>,
    pub mode: Option<String>,
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

fn error_redirect(page: &str, message: &str) -> Redirect {
    let separator = if page.contains('?') { '&' } else { '?' };
    Redirect::to(&format!("{page}{separator}sso_error={}", encode_component(message)))
}

pub async fn start(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
    Query(query): Query<StartQuery>,
    session_jar: SignedCookieJar,
    jar: PrivateCookieJar,
) -> Response {
    let settings = auth_settings::load(&state).await;
    let Some(provider) = state.sso.find(&provider_id).filter(|provider| settings.provider_enabled(&provider.id)).cloned() else {
        return error_redirect(LOGIN_PAGE, "This sign-in provider is not available").into_response();
    };
    let Some(redirect_uri) = provider.callback_url() else {
        return error_redirect(LOGIN_PAGE, "PUBLIC_URL is not set on the server").into_response();
    };

    let linking_user_id = if query.mode.as_deref() == Some("link") {
        match session_jar.get("session_user_id") {
            Some(cookie) => Some(cookie.value().to_string()),
            None => return error_redirect(LOGIN_PAGE, "Sign in before connecting another account").into_response(),
        }
    } else {
        None
    };

    let redirect_to = query
        .redirect
        .filter(|path| crate::site_url::is_safe_redirect(path))
        .unwrap_or_else(|| if linking_user_id.is_some() { ACCOUNT_PAGE.to_string() } else { "/drive".to_string() });
    let pending = PendingSignIn::new(&provider.id, redirect_to, linking_user_id);

    let authorization_url = match &provider.kind {
        ProviderKind::GitHub => github::authorization_url(&provider, &redirect_uri, &pending.state),
        ProviderKind::Oidc { issuer } => match state.sso.endpoints(issuer).await {
            Ok(endpoints) => oidc::authorization_url(&endpoints, &provider, &redirect_uri, &pending.state, &pending.nonce, &pending.code_challenge()),
            Err(error) => {
                tracing::warn!("Discovery for {} failed: {error}", provider.name);
                return error_redirect(LOGIN_PAGE, &format!("{} is not reachable right now", provider.name)).into_response();
            }
        },
    };

    (state_cookie::store(jar, &pending), Redirect::to(&authorization_url)).into_response()
}

async fn fetch_profile(state: &AppState, provider: &ProviderConfig, pending: &PendingSignIn, code: &str) -> Result<ExternalProfile, String> {
    let redirect_uri = provider.callback_url().ok_or("PUBLIC_URL is not set on the server")?;
    match &provider.kind {
        ProviderKind::GitHub => github::exchange_code(provider, &redirect_uri, code).await,
        ProviderKind::Oidc { issuer } => {
            let endpoints = state.sso.endpoints(issuer).await?;
            oidc::exchange_code(&endpoints, provider, &redirect_uri, code, &pending.code_verifier, &pending.nonce).await
        }
    }
}

pub async fn callback(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
    Query(query): Query<CallbackQuery>,
    client: ClientInfo,
    jar: PrivateCookieJar,
    session_jar: CookieJar,
) -> Response {
    let (jar, pending) = state_cookie::take(jar);
    let Some(pending) = pending.filter(|pending| pending.provider_id == provider_id) else {
        return (jar, error_redirect(LOGIN_PAGE, "The sign-in request expired. Please try again.")).into_response();
    };
    let failure_page = if pending.linking_user_id.is_some() { ACCOUNT_PAGE } else { LOGIN_PAGE };

    if let Some(error) = query.error {
        let message = query.error_description.unwrap_or(error);
        return (jar, error_redirect(failure_page, &format!("Sign-in was cancelled: {message}"))).into_response();
    }
    if query.state.as_deref() != Some(pending.state.as_str()) {
        return (jar, error_redirect(failure_page, "The sign-in response did not match this request")).into_response();
    }
    let Some(code) = query.code else {
        return (jar, error_redirect(failure_page, "The provider did not return a sign-in code")).into_response();
    };

    let settings = auth_settings::load(&state).await;
    let Some(provider) = state.sso.find(&provider_id).filter(|provider| settings.provider_enabled(&provider.id)).cloned() else {
        return (jar, error_redirect(failure_page, "This sign-in provider is not available")).into_response();
    };

    let profile = match fetch_profile(&state, &provider, &pending, &code).await {
        Ok(profile) => profile,
        Err(error) => {
            tracing::warn!("{} sign-in failed: {error}", provider.name);
            return (jar, error_redirect(failure_page, &format!("{} sign-in failed: {error}", provider.name))).into_response();
        }
    };

    if let Some(user_id) = &pending.linking_user_id {
        return match linking::link_to_user(&state, user_id, &provider, &profile).await {
            Ok(()) => (jar, Redirect::to(&pending.redirect_to)).into_response(),
            Err(error) => (jar, error_redirect(ACCOUNT_PAGE, &error)).into_response(),
        };
    }

    let user_id = match linking::resolve_account(&state, &provider, &profile, &settings).await {
        Ok(user_id) => user_id,
        Err(error) => return (jar, error_redirect(LOGIN_PAGE, &error)).into_response(),
    };

    match sessions::start_session(&state, &user_id, &client).await {
        Ok(cookie) => (jar, session_jar.add(cookie), Redirect::to(&pending.redirect_to)).into_response(),
        Err((_, message)) => (jar, error_redirect(LOGIN_PAGE, &message)).into_response(),
    }
}

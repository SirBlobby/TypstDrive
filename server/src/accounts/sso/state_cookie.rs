use axum_extra::extract::cookie::{Cookie, PrivateCookieJar, SameSite};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

const COOKIE_NAME: &str = "typstdrive_sso";
const COOKIE_PATH: &str = "/api/auth/sso";
const LIFETIME_MINUTES: i64 = 10;

#[derive(Serialize, Deserialize)]
pub struct PendingSignIn {
    pub provider_id: String,
    pub state: String,
    pub nonce: String,
    pub code_verifier: String,
    pub redirect_to: String,
    pub linking_user_id: Option<String>,
}

fn random_value() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

impl PendingSignIn {
    pub fn new(provider_id: &str, redirect_to: String, linking_user_id: Option<String>) -> Self {
        Self {
            provider_id: provider_id.to_string(),
            state: random_value(),
            nonce: random_value(),
            code_verifier: random_value(),
            redirect_to,
            linking_user_id,
        }
    }

    pub fn code_challenge(&self) -> String {
        URL_SAFE_NO_PAD.encode(Sha256::digest(self.code_verifier.as_bytes()).as_slice())
    }
}

pub fn store(jar: PrivateCookieJar, pending: &PendingSignIn) -> PrivateCookieJar {
    let value = serde_json::to_string(pending).unwrap_or_default();
    let mut cookie = Cookie::new(COOKIE_NAME, value);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path(COOKIE_PATH);
    cookie.set_secure(crate::site_url::uses_https());
    cookie.set_max_age(cookie::time::Duration::minutes(LIFETIME_MINUTES));
    jar.add(cookie)
}

pub fn take(jar: PrivateCookieJar) -> (PrivateCookieJar, Option<PendingSignIn>) {
    let pending = jar.get(COOKIE_NAME).and_then(|cookie| serde_json::from_str(cookie.value()).ok());
    let jar = jar.remove(Cookie::build(COOKIE_NAME).path(COOKIE_PATH).build());
    (jar, pending)
}

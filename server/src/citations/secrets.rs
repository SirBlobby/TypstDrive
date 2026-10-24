use axum_extra::extract::cookie::{Cookie, Key};

const SEALED_NAME: &str = "zotero_key";

pub fn seal(key: &Key, value: &str) -> String {
    let mut jar = cookie::CookieJar::new();
    jar.private_mut(key).add(Cookie::new(SEALED_NAME, value.to_string()));
    jar.get(SEALED_NAME).map(|cookie| cookie.value().to_string()).unwrap_or_default()
}

pub fn open(key: &Key, sealed: &str) -> Option<String> {
    let mut jar = cookie::CookieJar::new();
    jar.add_original(Cookie::new(SEALED_NAME, sealed.to_string()));
    jar.private(key).get(SEALED_NAME).map(|cookie| cookie.value().to_string())
}

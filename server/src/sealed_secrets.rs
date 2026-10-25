use axum_extra::extract::cookie::{Cookie, Key};

pub fn seal(key: &Key, name: &str, value: &str) -> String {
    let mut jar = cookie::CookieJar::new();
    jar.private_mut(key).add(Cookie::new(name.to_string(), value.to_string()));
    jar.get(name).map(|cookie| cookie.value().to_string()).unwrap_or_default()
}

pub fn open(key: &Key, name: &str, sealed: &str) -> Option<String> {
    let mut jar = cookie::CookieJar::new();
    jar.add_original(Cookie::new(name.to_string(), sealed.to_string()));
    jar.private(key).get(name).map(|cookie| cookie.value().to_string())
}

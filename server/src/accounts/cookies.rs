use axum::http::header::COOKIE;
use axum::http::{HeaderMap, HeaderValue};
use axum_extra::extract::cookie::{Cookie, Key};

pub const USER_ID_COOKIE: &str = "session_user_id";

fn cookie_pairs(headers: &HeaderMap) -> Vec<(String, String)> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| {
            let (name, value) = pair.trim().split_once('=')?;
            Some((name.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

pub fn read(headers: &HeaderMap, name: &str) -> Option<String> {
    cookie_pairs(headers)
        .into_iter()
        .find(|(cookie_name, _)| cookie_name == name)
        .map(|(_, value)| value)
        .filter(|value| !value.is_empty())
}

fn signed_value(key: &Key, name: &str, value: &str) -> Option<String> {
    let mut jar = cookie::CookieJar::new();
    jar.signed_mut(key).add(Cookie::new(name.to_string(), value.to_string()));
    jar.get(name).map(|cookie| cookie.value().to_string())
}

pub fn replace_signed(headers: &mut HeaderMap, key: &Key, name: &str, value: Option<&str>) {
    let mut kept: Vec<String> = cookie_pairs(headers)
        .into_iter()
        .filter(|(cookie_name, _)| cookie_name != name)
        .map(|(cookie_name, cookie_value)| format!("{cookie_name}={cookie_value}"))
        .collect();

    if let Some(signed) = value.and_then(|value| signed_value(key, name, value)) {
        kept.push(format!("{name}={signed}"));
    }

    headers.remove(COOKIE);
    if kept.is_empty() {
        return;
    }
    if let Ok(header_value) = HeaderValue::from_str(&kept.join("; ")) {
        headers.insert(COOKIE, header_value);
    }
}

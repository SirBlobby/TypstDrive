use std::net::SocketAddr;

use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::request::Parts;
use axum::http::HeaderMap;

pub struct ClientInfo {
    pub ip_address: String,
    pub user_agent: String,
}

fn trusts_proxy_headers() -> bool {
    std::env::var("TRUST_PROXY_HEADERS").map(|value| value.eq_ignore_ascii_case("true") || value == "1").unwrap_or(false)
}

fn forwarded_address(headers: &HeaderMap) -> Option<String> {
    let forwarded_for = headers.get("x-forwarded-for").and_then(|value| value.to_str().ok());
    let real_ip = headers.get("x-real-ip").and_then(|value| value.to_str().ok());
    forwarded_for
        .and_then(|value| value.split(',').map(str::trim).filter(|part| !part.is_empty()).last())
        .or(real_ip)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn client_ip(headers: &HeaderMap, socket: Option<SocketAddr>) -> String {
    if trusts_proxy_headers() {
        if let Some(address) = forwarded_address(headers) {
            return address;
        }
    }
    socket.map(|address| address.ip().to_string()).unwrap_or_else(|| "unknown".to_string())
}

pub fn user_agent(headers: &HeaderMap) -> String {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.chars().take(300).collect())
        .unwrap_or_default()
}

impl<S: Send + Sync> FromRequestParts<S> for ClientInfo {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let socket = parts.extensions.get::<ConnectInfo<SocketAddr>>().map(|info| info.0);
        Ok(ClientInfo {
            ip_address: client_ip(&parts.headers, socket),
            user_agent: user_agent(&parts.headers),
        })
    }
}

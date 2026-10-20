use std::time::Duration;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde_json::Value;

const REQUEST_TIMEOUT_SECONDS: u64 = 15;
const USER_AGENT: &str = "TypstDrive";

pub fn encode_component(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (byte as char).to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

pub fn query_string(pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .map(|(name, value)| format!("{}={}", encode_component(name), encode_component(value)))
        .collect::<Vec<_>>()
        .join("&")
}

pub fn basic_credentials(client_id: &str, client_secret: &str) -> String {
    let joined = format!("{}:{}", encode_component(client_id), encode_component(client_secret));
    format!("Basic {}", BASE64.encode(joined))
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .user_agent(USER_AGENT)
        .build()
}

fn read_json(result: Result<ureq::Response, ureq::Error>) -> Result<Value, String> {
    let response = match result {
        Ok(response) => response,
        Err(ureq::Error::Status(code, response)) => {
            let body = response.into_string().unwrap_or_default();
            return Err(format!("the provider answered with status {code}: {}", body.chars().take(300).collect::<String>()));
        }
        Err(error) => return Err(format!("could not reach the provider: {error}")),
    };
    let body = response.into_string().map_err(|error| error.to_string())?;
    serde_json::from_str(&body).map_err(|_| "the provider sent a response that is not JSON".to_string())
}

pub async fn get_json(url: String, bearer_token: Option<String>) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut request = agent().get(&url).set("Accept", "application/json");
        if let Some(token) = bearer_token {
            request = request.set("Authorization", &format!("Bearer {token}"));
        }
        read_json(request.call())
    })
    .await
    .map_err(|error| error.to_string())?
}

pub async fn post_form(url: String, form: Vec<(String, String)>, authorization: Option<String>) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut request = agent().post(&url).set("Accept", "application/json");
        if let Some(authorization) = authorization {
            request = request.set("Authorization", &authorization);
        }
        let pairs: Vec<(&str, &str)> = form.iter().map(|(name, value)| (name.as_str(), value.as_str())).collect();
        read_json(request.send_form(&pairs))
    })
    .await
    .map_err(|error| error.to_string())?
}

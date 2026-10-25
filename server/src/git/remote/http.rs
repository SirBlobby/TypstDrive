use std::io::Read;
use std::sync::OnceLock;
use std::time::Duration;

use axum::http::StatusCode;
use serde_json::Value;

use crate::git::file_rules::MAX_FILE_BYTES;
use crate::git::ApiError;
use crate::remote_hosts::resolve_remote_host;

const REQUEST_TIMEOUT_SECONDS: u64 = 120;
const MAX_ERROR_BODY_BYTES: u64 = 64 * 1024;
const MAX_MESSAGE_CHARACTERS: usize = 300;
const USER_AGENT: &str = "TypstDrive (Git sync)";

pub struct Client {
    agent: ureq::Agent,
    base: String,
    headers: Vec<(&'static str, String)>,
    service: &'static str,
}

fn remote_message(response: ureq::Response) -> String {
    let mut body = String::new();
    let _ = response.into_reader().take(MAX_ERROR_BODY_BYTES).read_to_string(&mut body);
    let parsed: Option<Value> = serde_json::from_str(&body).ok();
    let message = parsed.as_ref().and_then(|value| value.get("message").or_else(|| value.get("error")));
    let text = match message {
        Some(Value::String(text)) => text.clone(),
        Some(other) => other.to_string(),
        None => String::new(),
    };
    text.chars().filter(|character| !character.is_control()).take(MAX_MESSAGE_CHARACTERS).collect()
}

fn shared_agent() -> ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT
        .get_or_init(|| {
            ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
                .user_agent(USER_AGENT)
                .redirects(0)
                .resolver(resolve_remote_host)
                .build()
        })
        .clone()
}

fn status_error(service: &str, code: u16, message: String) -> ApiError {
    let detail = if message.is_empty() { String::new() } else { format!(" ({message})") };
    match code {
        401 => (StatusCode::FORBIDDEN, format!("{service} did not accept the access token. Update the token in the Git settings.")),
        403 => (StatusCode::FORBIDDEN, format!("{service} refused access{detail}. The token needs read and write access to this repository.")),
        404 => (StatusCode::NOT_FOUND, format!("{service} could not find the repository, branch or folder. Private repositories need an access token.")),
        409 | 422 => (StatusCode::CONFLICT, format!("{service} rejected the change{detail}. Pull the latest changes and try again.")),
        429 => (StatusCode::TOO_MANY_REQUESTS, format!("{service} is rate limiting requests. Try again in a minute.")),
        _ => (StatusCode::BAD_GATEWAY, format!("{service} answered with status {code}{detail}")),
    }
}

impl Client {
    pub fn new(service: &'static str, base: String, headers: Vec<(&'static str, String)>) -> Self {
        Self { agent: shared_agent(), base, headers, service }
    }

    fn request(&self, method: &str, path: &str) -> ureq::Request {
        self.headers
            .iter()
            .fold(self.agent.request(method, &format!("{}{path}", self.base)), |request, (name, value)| request.set(name, value))
    }

    fn finish(&self, result: Result<ureq::Response, ureq::Error>) -> Result<ureq::Response, ApiError> {
        match result {
            Ok(response) if (300..400).contains(&response.status()) => Err((
                StatusCode::BAD_GATEWAY,
                format!("{} redirected the request. Use the current address of the repository or server.", self.service),
            )),
            Ok(response) => Ok(response),
            Err(ureq::Error::Status(code, response)) => Err(status_error(self.service, code, remote_message(response))),
            Err(ureq::Error::Transport(error)) => Err((StatusCode::BAD_GATEWAY, format!("Could not reach {}: {error}", self.service))),
        }
    }

    fn unreadable(&self) -> ApiError {
        (StatusCode::BAD_GATEWAY, format!("{} sent an unreadable answer", self.service))
    }

    pub fn get(&self, path: &str, extra_headers: &[(&str, &str)]) -> Result<ureq::Response, ApiError> {
        let request = extra_headers.iter().fold(self.request("GET", path), |request, (name, value)| request.set(name, value));
        self.finish(request.call())
    }

    pub fn head(&self, path: &str) -> Result<ureq::Response, ApiError> {
        self.finish(self.request("HEAD", path).call())
    }

    pub fn get_json(&self, path: &str) -> Result<Value, ApiError> {
        self.get(path, &[])?.into_json::<Value>().map_err(|_| self.unreadable())
    }

    pub fn send_json(&self, method: &str, path: &str, body: &Value) -> Result<Value, ApiError> {
        self.finish(self.request(method, path).send_json(body))?
            .into_json::<Value>()
            .map_err(|_| self.unreadable())
    }

    pub fn read_limited(&self, response: ureq::Response) -> Result<Option<Vec<u8>>, ApiError> {
        let mut bytes = Vec::new();
        response
            .into_reader()
            .take(MAX_FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| self.unreadable())?;
        Ok((bytes.len() as u64 <= MAX_FILE_BYTES).then_some(bytes))
    }
}

pub fn text_field(value: &Value, pointer: &str) -> Option<String> {
    value.pointer(pointer).and_then(Value::as_str).map(str::to_string)
}

pub fn encode_segment(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (byte as char).to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_path_segments() {
        assert_eq!(encode_segment("group/sub repo"), "group%2Fsub%20repo");
        assert_eq!(encode_segment("main.typ"), "main.typ");
    }
}

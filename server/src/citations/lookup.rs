use std::time::Duration;

use axum::http::StatusCode;
use serde_json::Value;

use super::ApiError;

const REQUEST_TIMEOUT_SECONDS: u64 = 15;
const CSL_JSON: &str = "application/vnd.citationstyles.csl+json";
const ARXIV_DOI_PREFIX: &str = "10.48550/arXiv.";

#[derive(Debug, PartialEq, Eq)]
pub enum Identifier {
    Doi(String),
    Arxiv(String),
}

impl Identifier {
    pub fn doi(&self) -> String {
        match self {
            Self::Doi(doi) => doi.clone(),
            Self::Arxiv(id) => format!("{ARXIV_DOI_PREFIX}{id}"),
        }
    }
}

fn strip_prefix_ignore_case<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let head = text.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix).then(|| &text[prefix.len()..])
}

fn without_version(id: &str) -> &str {
    match id.rfind('v') {
        Some(position) if position > 0 && id[position + 1..].chars().all(|c| c.is_ascii_digit()) && position + 1 < id.len() => &id[..position],
        _ => id,
    }
}

fn is_new_arxiv_id(id: &str) -> bool {
    let Some((prefix, number)) = id.split_once('.') else {
        return false;
    };
    prefix.len() == 4 && prefix.chars().all(|c| c.is_ascii_digit()) && (4..=5).contains(&number.len()) && number.chars().all(|c| c.is_ascii_digit())
}

fn is_old_arxiv_id(id: &str) -> bool {
    let Some((archive, number)) = id.split_once('/') else {
        return false;
    };
    !archive.is_empty()
        && archive.chars().all(|c| c.is_ascii_lowercase() || c == '-' || c == '.')
        && number.len() == 7
        && number.chars().all(|c| c.is_ascii_digit())
}

fn arxiv_id(candidate: &str) -> Option<String> {
    let trimmed = candidate.trim_end_matches(".pdf").trim_end_matches('/');
    let id = without_version(trimmed);
    (is_new_arxiv_id(id) || is_old_arxiv_id(id)).then(|| id.to_string())
}

pub fn parse_identifier(input: &str) -> Result<Identifier, ApiError> {
    let invalid = || (StatusCode::BAD_REQUEST, "Enter a DOI such as 10.1000/xyz123 or an arXiv ID such as 2301.00001".to_string());
    let mut text = input.trim();
    for prefix in ["https://", "http://"] {
        if let Some(rest) = strip_prefix_ignore_case(text, prefix) {
            text = rest;
        }
    }
    if let Some(rest) = strip_prefix_ignore_case(text, "www.") {
        text = rest;
    }

    for prefix in ["doi.org/", "dx.doi.org/", "doi:"] {
        if let Some(rest) = strip_prefix_ignore_case(text, prefix) {
            return parse_doi(rest.trim()).ok_or_else(invalid);
        }
    }
    for prefix in ["arxiv.org/abs/", "arxiv.org/pdf/", "export.arxiv.org/abs/", "arxiv:"] {
        if let Some(rest) = strip_prefix_ignore_case(text, prefix) {
            return arxiv_id(rest.trim()).map(Identifier::Arxiv).ok_or_else(invalid);
        }
    }
    if let Some(doi) = parse_doi(text) {
        let doi_text = doi.doi();
        if let Some(id) = strip_prefix_ignore_case(&doi_text, ARXIV_DOI_PREFIX).and_then(arxiv_id) {
            return Ok(Identifier::Arxiv(id));
        }
        return Ok(doi);
    }
    arxiv_id(text).map(Identifier::Arxiv).ok_or_else(invalid)
}

fn parse_doi(text: &str) -> Option<Identifier> {
    let decoded = text.replace("%2F", "/").replace("%2f", "/");
    let doi = decoded.trim().trim_end_matches(['.', ',', ';']);
    let (prefix, suffix) = doi.split_once('/')?;
    let valid_prefix = prefix.starts_with("10.") && prefix[3..].split('.').all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()));
    let valid_suffix = !suffix.is_empty() && !suffix.chars().any(|c| c.is_whitespace() || c.is_control());
    (valid_prefix && valid_suffix).then(|| Identifier::Doi(doi.to_string()))
}

fn encode_doi(doi: &str) -> String {
    doi.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b'(' | b')' | b':' | b';' => (byte as char).to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

pub async fn fetch_csl(identifier: &Identifier) -> Result<Value, ApiError> {
    let url = format!("https://doi.org/{}", encode_doi(&identifier.doi()));
    let mut item = tokio::task::spawn_blocking(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
            .user_agent("TypstDrive (citation lookup)")
            .redirects(5)
            .build();
        match agent.get(&url).set("Accept", CSL_JSON).call() {
            Ok(response) => response
                .into_json::<Value>()
                .map_err(|_| (StatusCode::BAD_GATEWAY, "The citation service sent an unreadable answer".to_string())),
            Err(ureq::Error::Status(404, _)) => Err((StatusCode::NOT_FOUND, "No publication was found for that identifier".to_string())),
            Err(ureq::Error::Status(code, _)) => Err((StatusCode::BAD_GATEWAY, format!("The citation service answered with status {code}"))),
            Err(_) => Err((StatusCode::BAD_GATEWAY, "Could not reach the citation service".to_string())),
        }
    })
    .await
    .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))??;

    if !item.is_object() {
        return Err((StatusCode::BAD_GATEWAY, "The citation service sent an unreadable answer".to_string()));
    }
    if let (Identifier::Arxiv(id), Some(object)) = (identifier, item.as_object_mut()) {
        object.insert("arxiv".to_string(), Value::String(id.clone()));
        object.entry("URL".to_string()).or_insert_with(|| Value::String(format!("https://arxiv.org/abs/{id}")));
    }
    Ok(item)
}

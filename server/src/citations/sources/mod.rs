mod doi;
mod pubmed;
mod semantic_scholar;

use std::time::Duration;

use axum::http::StatusCode;
use serde_json::Value;

use super::identifiers::{Identifier, ARXIV_DOI_PREFIX};
use super::ApiError;

const REQUEST_TIMEOUT_SECONDS: u64 = 15;
const USER_AGENT: &str = "TypstDrive (citation lookup)";

pub struct JsonRequest {
    pub url: String,
    pub accept: &'static str,
    pub service: &'static str,
    pub api_key: Option<(&'static str, String)>,
}

impl JsonRequest {
    pub fn new(url: String, accept: &'static str, service: &'static str) -> Self {
        Self { url, accept, service, api_key: None }
    }
}

pub fn not_found() -> ApiError {
    (StatusCode::NOT_FOUND, "No publication was found for that identifier".to_string())
}

fn unreadable(service: &str) -> ApiError {
    (StatusCode::BAD_GATEWAY, format!("{service} sent an unreadable answer"))
}

pub async fn get_json(request: JsonRequest) -> Result<Option<Value>, ApiError> {
    tokio::task::spawn_blocking(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
            .user_agent(USER_AGENT)
            .redirects(5)
            .build();
        let mut call = agent.get(&request.url).set("Accept", request.accept);
        if let Some((header, key)) = &request.api_key {
            call = call.set(header, key);
        }
        match call.call() {
            Ok(response) => response.into_json::<Value>().map(Some).map_err(|_| unreadable(request.service)),
            Err(ureq::Error::Status(404 | 400 | 410, _)) => Ok(None),
            Err(ureq::Error::Status(429, _)) => Err((StatusCode::TOO_MANY_REQUESTS, format!("{} is busy. Try again in a moment.", request.service))),
            Err(ureq::Error::Status(code, _)) => Err((StatusCode::BAD_GATEWAY, format!("{} answered with status {code}", request.service))),
            Err(_) => Err((StatusCode::BAD_GATEWAY, format!("Could not reach {}", request.service))),
        }
    })
    .await
    .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
}

pub fn arxiv_doi(id: &str) -> String {
    format!("{ARXIV_DOI_PREFIX}{id}")
}

pub fn mark_arxiv(item: &mut Value, id: &str) {
    if let Some(object) = item.as_object_mut() {
        object.insert("arxiv".to_string(), Value::String(id.to_string()));
        object.entry("URL".to_string()).or_insert_with(|| Value::String(format!("https://arxiv.org/abs/{id}")));
    }
}

pub async fn fetch_csl(identifier: &Identifier) -> Result<Value, ApiError> {
    let item = match identifier {
        Identifier::Doi(doi) => doi::fetch(doi).await?,
        Identifier::Arxiv(id) => {
            let mut item = doi::fetch(&arxiv_doi(id)).await?;
            mark_arxiv(&mut item, id);
            item
        }
        Identifier::Pubmed(id) => pubmed::fetch(pubmed::Database::Pubmed, id).await?,
        Identifier::Pmc(id) => pubmed::fetch(pubmed::Database::Pmc, id).await?,
        Identifier::SemanticScholar(id) => semantic_scholar::fetch(id).await?,
    };
    if !item.is_object() {
        return Err(unreadable("The citation service"));
    }
    Ok(item)
}

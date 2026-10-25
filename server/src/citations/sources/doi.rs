use serde_json::Value;

use super::{get_json, not_found, JsonRequest};
use crate::citations::ApiError;

const CSL_JSON: &str = "application/vnd.citationstyles.csl+json";

fn encode_doi(doi: &str) -> String {
    doi.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b'(' | b')' | b':' | b';' => (byte as char).to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

async fn from_crossref(encoded: &str) -> Result<Option<Value>, ApiError> {
    let url = format!("https://api.crossref.org/v1/works/{encoded}/transform/{CSL_JSON}");
    get_json(JsonRequest::new(url, CSL_JSON, "Crossref")).await
}

pub async fn fetch(doi: &str) -> Result<Value, ApiError> {
    let encoded = encode_doi(doi);
    let resolver = get_json(JsonRequest::new(format!("https://doi.org/{encoded}"), CSL_JSON, "doi.org")).await;
    match resolver {
        Ok(Some(item)) => Ok(item),
        Ok(None) => from_crossref(&encoded).await.ok().flatten().ok_or_else(not_found),
        Err(problem) => from_crossref(&encoded).await.ok().flatten().ok_or(problem),
    }
}

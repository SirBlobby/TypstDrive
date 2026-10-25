use serde_json::Value;

use super::{get_json, not_found, JsonRequest};
use crate::citations::ApiError;

#[derive(Clone, Copy)]
pub enum Database {
    Pubmed,
    Pmc,
}

impl Database {
    fn path(self) -> &'static str {
        match self {
            Self::Pubmed => "pubmed",
            Self::Pmc => "pmc",
        }
    }
}

fn first_item(answer: Value) -> Option<Value> {
    let item = match answer {
        Value::Array(items) => items.into_iter().next()?,
        other => other,
    };
    item.get("title").is_some().then_some(item)
}

pub async fn fetch(database: Database, id: &str) -> Result<Value, ApiError> {
    let url = format!("https://api.ncbi.nlm.nih.gov/lit/ctxp/v1/{}/?format=csl&id={id}", database.path());
    let answer = get_json(JsonRequest::new(url, "application/json", "PubMed")).await?.ok_or_else(not_found)?;
    let mut item = first_item(answer).ok_or_else(not_found)?;
    if let (Database::Pmc, Some(object)) = (database, item.as_object_mut()) {
        object.entry("PMCID".to_string()).or_insert_with(|| Value::String(format!("PMC{id}")));
    }
    Ok(item)
}

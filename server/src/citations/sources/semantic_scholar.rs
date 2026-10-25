use serde_json::{json, Map, Value};

use super::{arxiv_doi, doi, get_json, mark_arxiv, not_found, JsonRequest};
use crate::citations::ApiError;

const FIELDS: &str = "title,authors,year,venue,journal,externalIds,publicationDate,url,publicationTypes";
const API_KEY_VARIABLE: &str = "SEMANTIC_SCHOLAR_API_KEY";

fn text<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str).map(str::trim).filter(|text| !text.is_empty())
}

fn csl_type(paper: &Value) -> &'static str {
    let types: Vec<&str> = paper
        .get("publicationTypes")
        .and_then(Value::as_array)
        .map(|list| list.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    if types.contains(&"Conference") {
        "paper-conference"
    } else if types.contains(&"JournalArticle") || types.contains(&"Review") {
        "article-journal"
    } else if types.contains(&"Book") {
        "book"
    } else if types.contains(&"BookSection") {
        "chapter"
    } else {
        "article"
    }
}

fn person(name: &str) -> Value {
    match name.trim().rsplit_once(' ') {
        Some((given, family)) => json!({ "given": given, "family": family }),
        None => json!({ "literal": name.trim() }),
    }
}

fn issued(paper: &Value) -> Option<Value> {
    let parts: Vec<i64> = match text(paper, "publicationDate") {
        Some(date) => date.split('-').filter_map(|part| part.parse().ok()).collect(),
        None => vec![paper.get("year")?.as_i64()?],
    };
    (!parts.is_empty()).then(|| json!({ "date-parts": [parts] }))
}

fn to_csl(paper: &Value) -> Value {
    let mut item = Map::new();
    item.insert("type".into(), json!(csl_type(paper)));
    if let Some(title) = text(paper, "title") {
        item.insert("title".into(), json!(title));
    }
    let authors: Vec<Value> = paper
        .get("authors")
        .and_then(Value::as_array)
        .map(|list| list.iter().filter_map(|author| text(author, "name")).map(person).collect())
        .unwrap_or_default();
    if !authors.is_empty() {
        item.insert("author".into(), Value::Array(authors));
    }
    let journal = paper.get("journal").cloned().unwrap_or(Value::Null);
    if let Some(container) = text(&journal, "name").or_else(|| text(paper, "venue")) {
        item.insert("container-title".into(), json!(container));
    }
    if let Some(volume) = text(&journal, "volume") {
        item.insert("volume".into(), json!(volume));
    }
    if let Some(pages) = text(&journal, "pages") {
        item.insert("page".into(), json!(pages));
    }
    if let Some(date) = issued(paper) {
        item.insert("issued".into(), date);
    }
    if let Some(url) = text(paper, "url") {
        item.insert("URL".into(), json!(url));
    }
    if let Some(pmid) = paper.get("externalIds").and_then(|ids| text(ids, "PubMed")) {
        item.insert("PMID".into(), json!(pmid));
    }
    Value::Object(item)
}

pub async fn fetch(id: &str) -> Result<Value, ApiError> {
    let mut request = JsonRequest::new(
        format!("https://api.semanticscholar.org/graph/v1/paper/{id}?fields={FIELDS}"),
        "application/json",
        "Semantic Scholar",
    );
    request.api_key = std::env::var(API_KEY_VARIABLE).ok().filter(|key| !key.is_empty()).map(|key| ("x-api-key", key));
    let paper = get_json(request).await?.ok_or_else(not_found)?;
    let external_ids = paper.get("externalIds").cloned().unwrap_or(Value::Null);

    if let Some(doi_name) = text(&external_ids, "DOI") {
        if let Ok(item) = doi::fetch(doi_name).await {
            return Ok(item);
        }
    }
    if let Some(arxiv_id) = text(&external_ids, "ArXiv") {
        if let Ok(mut item) = doi::fetch(&arxiv_doi(arxiv_id)).await {
            mark_arxiv(&mut item, arxiv_id);
            return Ok(item);
        }
    }
    if text(&paper, "title").is_none() {
        return Err(not_found());
    }
    Ok(to_csl(&paper))
}

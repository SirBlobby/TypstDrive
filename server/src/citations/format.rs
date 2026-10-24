use std::collections::HashSet;

use axum::http::StatusCode;
use serde_json::Value;

use super::csl::{base_key, unique_key, CslItem};
use super::{bibtex, hayagriva, ApiError};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BibliographyFormat {
    Bibtex,
    Hayagriva,
}

impl BibliographyFormat {
    pub fn parse(value: &str) -> Result<Self, ApiError> {
        match value {
            "bibtex" => Ok(Self::Bibtex),
            "hayagriva" => Ok(Self::Hayagriva),
            _ => Err((StatusCode::BAD_REQUEST, "Format must be bibtex or hayagriva".to_string())),
        }
    }

    pub fn for_file(file_name: &str) -> Result<Self, ApiError> {
        let lower = file_name.to_ascii_lowercase();
        if lower.ends_with(".bib") {
            Ok(Self::Bibtex)
        } else if lower.ends_with(".yml") || lower.ends_with(".yaml") {
            Ok(Self::Hayagriva)
        } else {
            Err((StatusCode::BAD_REQUEST, "Bibliography files must end in .bib, .yml or .yaml".to_string()))
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bibtex => "bibtex",
            Self::Hayagriva => "hayagriva",
        }
    }

    pub fn existing_keys(self, text: &str) -> HashSet<String> {
        let keys = match self {
            Self::Bibtex => bibtex::existing_keys(text),
            Self::Hayagriva => hayagriva::existing_keys(text),
        };
        keys.into_iter().collect()
    }

    fn render(self, key: &str, item: &CslItem) -> String {
        match self {
            Self::Bibtex => bibtex::entry(key, item),
            Self::Hayagriva => hayagriva::entry(key, item),
        }
    }
}

pub struct RenderedEntry {
    pub key: String,
    pub text: String,
}

pub fn render_entry(format: BibliographyFormat, item: &Value, taken: &mut HashSet<String>) -> RenderedEntry {
    let item = CslItem::new(item);
    let key = unique_key(&base_key(&item), taken);
    let text = format.render(&key, &item);
    RenderedEntry { key, text }
}

pub fn render_library(format: BibliographyFormat, items: &[Value]) -> (String, usize) {
    let mut ordered: Vec<(String, &Value)> = items.iter().map(|item| (base_key(&CslItem::new(item)), item)).collect();
    ordered.sort_by(|first, second| first.0.cmp(&second.0).then_with(|| stable_id(first.1).cmp(&stable_id(second.1))));

    let mut taken = HashSet::new();
    let entries: Vec<String> = ordered
        .into_iter()
        .map(|(_, item)| render_entry(format, item, &mut taken).text)
        .collect();
    (entries.join("\n"), entries.len())
}

fn stable_id(item: &Value) -> String {
    item.get("id").map(Value::to_string).unwrap_or_default()
}

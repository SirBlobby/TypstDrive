use std::collections::HashSet;

use serde_json::Value;

const SKIPPED_TITLE_WORDS: &[&str] = &["a", "an", "the", "on", "of", "in", "for", "and", "to", "with", "towards", "toward"];

pub struct Person {
    pub family: String,
    pub given: Option<String>,
}

impl Person {
    pub fn display(&self) -> String {
        match &self.given {
            Some(given) => format!("{}, {}", self.family, given),
            None => self.family.clone(),
        }
    }
}

pub struct CslItem<'a> {
    value: &'a Value,
}

impl<'a> CslItem<'a> {
    pub fn new(value: &'a Value) -> Self {
        Self { value }
    }

    pub fn text(&self, field: &str) -> Option<String> {
        let value = self.value.get(field)?;
        let text = match value {
            Value::String(text) => text.clone(),
            Value::Number(number) => number.to_string(),
            Value::Array(items) => items.first().and_then(Value::as_str)?.to_string(),
            _ => return None,
        };
        let cleaned = clean_text(&text);
        (!cleaned.is_empty()).then_some(cleaned)
    }

    pub fn kind(&self) -> String {
        self.text("type").unwrap_or_else(|| "article".to_string())
    }

    pub fn people(&self, field: &str) -> Vec<Person> {
        let Some(list) = self.value.get(field).and_then(Value::as_array) else {
            return Vec::new();
        };
        list.iter()
            .filter_map(|person| {
                let family = person.get("family").and_then(Value::as_str).map(clean_text);
                let given = person.get("given").and_then(Value::as_str).map(clean_text).filter(|given| !given.is_empty());
                match family.filter(|family| !family.is_empty()) {
                    Some(family) => Some(Person { family, given }),
                    None => {
                        let literal = person.get("literal").and_then(Value::as_str).map(clean_text)?;
                        (!literal.is_empty()).then_some(Person { family: literal, given: None })
                    }
                }
            })
            .collect()
    }

    pub fn date_parts(&self) -> Vec<i64> {
        for field in ["issued", "published-print", "published-online", "created"] {
            let parts: Vec<i64> = self
                .value
                .get(field)
                .and_then(|date| date.get("date-parts"))
                .and_then(Value::as_array)
                .and_then(|outer| outer.first())
                .and_then(Value::as_array)
                .map(|inner| {
                    inner
                        .iter()
                        .filter_map(|part| part.as_i64().or_else(|| part.as_str().and_then(|text| text.trim().parse().ok())))
                        .collect()
                })
                .unwrap_or_default();
            if !parts.is_empty() {
                return parts;
            }
        }
        Vec::new()
    }

    pub fn year(&self) -> Option<i64> {
        self.date_parts().first().copied()
    }

    pub fn citation_key(&self) -> Option<String> {
        self.text("citation-key").or_else(|| self.text("citationKey")).filter(|key| is_valid_key(key))
    }

    pub fn arxiv_id(&self) -> Option<String> {
        let doi = self.text("DOI").unwrap_or_default();
        let lower = doi.to_ascii_lowercase();
        if let Some(position) = lower.find("arxiv.") {
            return Some(doi[position + "arxiv.".len()..].to_string());
        }
        self.text("arxiv")
    }
}

fn clean_text(text: &str) -> String {
    let without_tags = strip_markup(text);
    without_tags.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn strip_markup(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut inside_tag = false;
    for character in text.chars() {
        match character {
            '<' => inside_tag = true,
            '>' if inside_tag => inside_tag = false,
            _ if !inside_tag => result.push(character),
            _ => {}
        }
    }
    result.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"")
}

fn fold_character(character: char) -> Option<&'static str> {
    let folded = match character {
        'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' | 'ā' | 'ą' => "a",
        'ç' | 'ć' | 'č' => "c",
        'ď' | 'đ' => "d",
        'é' | 'è' | 'ê' | 'ë' | 'ē' | 'ę' | 'ě' => "e",
        'í' | 'ì' | 'î' | 'ï' | 'ī' => "i",
        'ł' => "l",
        'ñ' | 'ń' | 'ň' => "n",
        'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ø' | 'ō' | 'ő' => "o",
        'ř' => "r",
        'ś' | 'š' | 'ş' => "s",
        'ß' => "ss",
        'ť' => "t",
        'ú' | 'ù' | 'û' | 'ü' | 'ū' | 'ů' | 'ű' => "u",
        'ý' | 'ÿ' => "y",
        'ź' | 'ż' | 'ž' => "z",
        'æ' => "ae",
        'œ' => "oe",
        _ => return None,
    };
    Some(folded)
}

fn key_part(text: &str) -> String {
    let mut result = String::new();
    for character in text.to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            result.push(character);
        } else if let Some(folded) = fold_character(character) {
            result.push_str(folded);
        }
    }
    result
}

pub fn is_valid_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= 100 && key.chars().all(|character| character.is_ascii_alphanumeric() || "-_:.".contains(character))
}

pub fn base_key(item: &CslItem) -> String {
    if let Some(key) = item.citation_key() {
        return key;
    }
    let author = item
        .people("author")
        .into_iter()
        .next()
        .or_else(|| item.people("editor").into_iter().next())
        .map(|person| key_part(&person.family))
        .unwrap_or_default();
    let year = item.year().map(|year| year.to_string()).unwrap_or_default();
    let title_word = item
        .text("title")
        .unwrap_or_default()
        .split_whitespace()
        .map(key_part)
        .find(|word| !word.is_empty() && !SKIPPED_TITLE_WORDS.contains(&word.as_str()))
        .unwrap_or_default();
    let key = format!("{author}{year}{title_word}");
    if key.is_empty() { "ref".to_string() } else { key }
}

pub fn unique_key(base: &str, taken: &mut HashSet<String>) -> String {
    if taken.insert(base.to_string()) {
        return base.to_string();
    }
    for suffix in 'a'..='z' {
        let candidate = format!("{base}{suffix}");
        if taken.insert(candidate.clone()) {
            return candidate;
        }
    }
    let mut number = 2;
    loop {
        let candidate = format!("{base}-{number}");
        if taken.insert(candidate.clone()) {
            return candidate;
        }
        number += 1;
    }
}

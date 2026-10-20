use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static TRANSLATED: LazyLock<Mutex<HashMap<&'static str, &'static str>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn translated(query: &'static str) -> &'static str {
    let mut cache = TRANSLATED.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(existing) = cache.get(query) {
        return existing;
    }
    let leaked: &'static str = Box::leak(translate(query).into_boxed_str());
    cache.insert(query, leaked);
    leaked
}

pub fn translate(query: &str) -> String {
    let positional = positional_placeholders(query);
    let casts = positional.replace("AS BIGINT)", "AS SIGNED)");
    upsert_clause(&casts)
}

fn positional_placeholders(query: &str) -> String {
    let mut output = String::with_capacity(query.len());
    let mut characters = query.chars().peekable();
    let mut inside_quotes = false;
    let mut expected_number = 1;

    while let Some(character) = characters.next() {
        if character == '\'' {
            inside_quotes = !inside_quotes;
        }
        if character != '$' || inside_quotes || !characters.peek().is_some_and(|next| next.is_ascii_digit()) {
            output.push(character);
            continue;
        }

        let mut digits = String::new();
        while let Some(digit) = characters.peek().filter(|next| next.is_ascii_digit()) {
            digits.push(*digit);
            characters.next();
        }
        let number: usize = digits.parse().unwrap_or(0);
        assert_eq!(number, expected_number, "MySQL needs placeholders numbered once and in order: {query}");
        expected_number += 1;
        output.push('?');
    }
    output
}

fn upsert_clause(query: &str) -> String {
    let Some(start) = query.find("ON CONFLICT") else {
        return query.to_string();
    };
    let head = &query[..start];
    let after_keyword = &query[start + "ON CONFLICT".len()..];
    let (Some(open), Some(close)) = (after_keyword.find('('), after_keyword.find(')')) else {
        return query.to_string();
    };
    let conflict_columns = &after_keyword[open + 1..close];
    let action = after_keyword[close + 1..].trim_start();

    if let Some(rest) = action.strip_prefix("DO NOTHING") {
        let first_column = conflict_columns.split(',').next().unwrap_or("").trim();
        return format!("{head}ON DUPLICATE KEY UPDATE {first_column} = {first_column}{rest}");
    }
    if let Some(assignments) = action.strip_prefix("DO UPDATE SET") {
        return format!("{head}ON DUPLICATE KEY UPDATE{}", excluded_to_values(assignments));
    }
    query.to_string()
}

fn excluded_to_values(assignments: &str) -> String {
    let mut output = String::with_capacity(assignments.len());
    let mut remaining = assignments;
    while let Some(position) = remaining.find("excluded.") {
        output.push_str(&remaining[..position]);
        let after = &remaining[position + "excluded.".len()..];
        let name_length = after.find(|character: char| !(character.is_ascii_alphanumeric() || character == '_')).unwrap_or(after.len());
        output.push_str(&format!("VALUES({})", &after[..name_length]));
        remaining = &after[name_length..];
    }
    output.push_str(remaining);
    output
}

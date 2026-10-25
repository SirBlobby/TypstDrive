mod scan;

use serde::Deserialize;

use scan::{find_field, matching_close, split_top_level, value_end, without_comments, without_leading_comments};

#[derive(Deserialize, Clone)]
pub struct TemplateAuthor {
    pub name: String,
    pub given_names: Option<String>,
    pub family_name: Option<String>,
    pub affiliation: Option<String>,
    pub department: Option<String>,
    pub email: Option<String>,
    pub orcid: Option<String>,
}

enum AuthorField {
    Name,
    GivenNames,
    FamilyName,
    Affiliation,
    Department,
    Email,
    Orcid,
}

fn author_field(key: &str) -> Option<AuthorField> {
    let normalized = key.trim().trim_matches('"').to_ascii_lowercase().replace('_', "-");
    let field = match normalized.as_str() {
        "name" | "full-name" | "fullname" | "author" => AuthorField::Name,
        "given" | "given-name" | "given-names" | "first" | "first-name" | "firstname" => AuthorField::GivenNames,
        "family" | "family-name" | "last" | "last-name" | "lastname" | "surname" => AuthorField::FamilyName,
        "affiliation" | "affiliations" | "organization" | "organisation" | "institution" | "institute" | "university" | "org" | "company" => AuthorField::Affiliation,
        "department" | "dept" => AuthorField::Department,
        "email" | "mail" | "e-mail" => AuthorField::Email,
        "orcid" | "orcid-id" => AuthorField::Orcid,
        _ => return None,
    };
    Some(field)
}

impl TemplateAuthor {
    fn split_name(&self) -> (String, String) {
        match self.name.trim().rsplit_once(' ') {
            Some((given, family)) => (given.to_string(), family.to_string()),
            None => (String::new(), self.name.trim().to_string()),
        }
    }

    fn value(&self, field: &AuthorField) -> String {
        let (given, family) = self.split_name();
        match field {
            AuthorField::Name => self.name.trim().to_string(),
            AuthorField::GivenNames => self.given_names.clone().unwrap_or(given),
            AuthorField::FamilyName => self.family_name.clone().unwrap_or(family),
            AuthorField::Affiliation => self.affiliation.clone().unwrap_or_default(),
            AuthorField::Department => self.department.clone().unwrap_or_default(),
            AuthorField::Email => self.email.clone().unwrap_or_default(),
            AuthorField::Orcid => self.orcid.clone().unwrap_or_default(),
        }
    }
}

fn string_literal(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

fn content_block(text: &str) -> String {
    let escaped: String = text
        .chars()
        .flat_map(|character| {
            let needs_escape = matches!(character, '\\' | '#' | '*' | '_' | '[' | ']' | '$' | '@' | '<' | '>' | '~' | '`');
            needs_escape.then_some('\\').into_iter().chain(std::iter::once(character))
        })
        .collect();
    format!("[{escaped}]")
}

fn same_form(example: &str, text: &str) -> Option<String> {
    match example.as_bytes().first()? {
        b'"' => Some(string_literal(text)),
        b'[' => Some(content_block(text)),
        b'(' => {
            let inner = &example[1..example.len().saturating_sub(1)];
            let first = split_top_level(inner).into_iter().next()?;
            let item = same_form(first, text)?;
            Some(format!("({item},)"))
        }
        _ => None,
    }
}

fn dictionary_entries(dictionary: &str) -> Option<Vec<(String, String)>> {
    let inner = &dictionary[1..dictionary.len().checked_sub(1)?];
    split_top_level(inner)
        .into_iter()
        .map(|part| {
            let cleaned = without_comments(part);
            let (key, expression) = cleaned.split_once(':')?;
            Some((key.trim().to_string(), expression.trim().to_string()))
        })
        .collect()
}

fn render_dictionary(entries: &[(String, String)], author: &TemplateAuthor) -> String {
    let fields: Vec<String> = entries
        .iter()
        .map(|(key, expression)| {
            let filled = author_field(key).and_then(|field| same_form(expression, &author.value(&field)));
            format!("{key}: {}", filled.unwrap_or_else(|| expression.clone()))
        })
        .collect();
    format!("({})", fields.join(", "))
}

fn line_indent(text: &str, position: usize) -> &str {
    let line_start = text[..position].rfind('\n').map_or(0, |index| index + 1);
    let line = &text[line_start..];
    &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
}

fn render_list(items: Vec<String>, outer_indent: &str, item_indent: &str) -> String {
    let body: String = items.iter().map(|item| format!("{item_indent}{item},\n")).collect();
    format!("(\n{body}{outer_indent})")
}

fn fill_list(source: &str, value_start: usize, value: &str, authors: &[TemplateAuthor]) -> Option<String> {
    let inner = &value[1..value.len() - 1];
    let first = without_leading_comments(split_top_level(inner).into_iter().next()?);
    let outer_indent = line_indent(source, value_start);
    let first_offset = value_start + 1 + inner.find(first)?;
    let item_indent = if source[value_start..first_offset].contains('\n') {
        line_indent(source, first_offset).to_string()
    } else {
        format!("{outer_indent}  ")
    };

    let items: Vec<String> = match first.as_bytes().first()? {
        b'(' => {
            let close = matching_close(first, 0)?;
            let entries = dictionary_entries(&first[..=close])?;
            if !entries.iter().any(|(key, _)| matches!(author_field(key), Some(AuthorField::Name | AuthorField::FamilyName))) {
                return None;
            }
            authors.iter().map(|author| render_dictionary(&entries, author)).collect()
        }
        b'"' | b'[' => authors.iter().map(|author| same_form(first, author.name.trim())).collect::<Option<_>>()?,
        _ => return None,
    };
    Some(render_list(items, outer_indent, &item_indent))
}

fn joined_names(authors: &[TemplateAuthor]) -> String {
    authors.iter().map(|author| author.name.trim()).collect::<Vec<_>>().join(", ")
}

pub fn fill_authors(source: &str, authors: &[TemplateAuthor]) -> Option<String> {
    let authors: Vec<TemplateAuthor> = authors.iter().filter(|author| !author.name.trim().is_empty()).cloned().collect();
    if authors.is_empty() {
        return None;
    }
    let field = find_field(source)?;
    let value = &source[field.value_start..field.value_end];
    value_end(value, 0)?;

    let replacement = match value.as_bytes()[0] {
        b'(' => fill_list(source, field.value_start, value, &authors)?,
        b'"' => string_literal(&joined_names(&authors)),
        b'[' => content_block(&joined_names(&authors)),
        _ => return None,
    };
    Some(format!("{}{replacement}{}", &source[..field.value_start], &source[field.value_end..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn author(name: &str, affiliation: &str) -> TemplateAuthor {
        TemplateAuthor {
            name: name.to_string(),
            given_names: None,
            family_name: None,
            affiliation: Some(affiliation.to_string()),
            department: None,
            email: None,
            orcid: Some("0000-0002-1825-0097".to_string()),
        }
    }

    #[test]
    fn fills_dictionary_lists() {
        let source = "#show: paper.with(\n  title: \"Title\",\n  authors: (\n    (name: \"Jane Doe\", affiliation: \"Somewhere\", note: 1),\n    (name: \"John Roe\", affiliation: \"Elsewhere\", note: 2),\n  ),\n)\n";
        let filled = fill_authors(source, &[author("Ada Lovelace", "Analytical Society")]).unwrap();
        assert!(filled.contains("(name: \"Ada Lovelace\", affiliation: \"Analytical Society\", note: 1),"));
        assert!(!filled.contains("Jane Doe"));
        assert!(filled.contains("title: \"Title\""));
    }

    #[test]
    fn fills_plain_strings() {
        let source = "#show: letter.with(author: \"Your Name\", date: today)";
        let filled = fill_authors(source, &[author("Ada Lovelace", ""), author("Charles Babbage", "")]).unwrap();
        assert_eq!(filled, "#show: letter.with(author: \"Ada Lovelace, Charles Babbage\", date: today)");
    }

    #[test]
    fn fills_let_bindings_and_skips_unknown_shapes() {
        let source = "#let authors = (\"A\", \"B\")\n";
        let filled = fill_authors(source, &[author("Ada Lovelace", "")]).unwrap();
        assert!(filled.contains("\"Ada Lovelace\","));
        assert!(fill_authors("#show: cv.with(author: me)", &[author("Ada", "")]).is_none());
        assert!(fill_authors("No fields here.", &[author("Ada", "")]).is_none());
    }
}

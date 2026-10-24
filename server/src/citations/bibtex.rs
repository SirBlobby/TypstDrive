use super::csl::{CslItem, Person};

const MONTHS: [&str; 12] = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];

fn escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' | '%' | '#' | '$' | '_' => {
                escaped.push('\\');
                escaped.push(character);
            }
            '{' | '}' | '\\' => {}
            _ => escaped.push(character),
        }
    }
    escaped
}

fn names(people: &[Person]) -> Option<String> {
    (!people.is_empty()).then(|| people.iter().map(|person| escape(&person.display())).collect::<Vec<_>>().join(" and "))
}

fn entry_type(item: &CslItem) -> &'static str {
    match item.kind().as_str() {
        "article-journal" | "article-magazine" | "article-newspaper" | "review" => "article",
        "article" if item.text("container-title").is_some() => "article",
        "book" => "book",
        "chapter" | "entry-encyclopedia" | "entry-dictionary" => "incollection",
        "paper-conference" => "inproceedings",
        "thesis" => "phdthesis",
        "report" => "techreport",
        "manuscript" => "unpublished",
        _ => "misc",
    }
}

fn container_field(entry_type: &str) -> &'static str {
    match entry_type {
        "article" => "journal",
        "incollection" | "inproceedings" => "booktitle",
        _ => "howpublished",
    }
}

fn publisher_field(entry_type: &str) -> &'static str {
    match entry_type {
        "phdthesis" => "school",
        "techreport" => "institution",
        _ => "publisher",
    }
}

pub fn entry(key: &str, item: &CslItem) -> String {
    let kind = entry_type(item);
    let mut fields: Vec<(&str, String)> = Vec::new();
    let mut push = |name: &'static str, value: Option<String>| {
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            fields.push((name, value));
        }
    };

    push("author", names(&item.people("author")));
    push("editor", names(&item.people("editor")));
    push("title", item.text("title").map(|title| escape(&title)));
    push(container_field(kind), item.text("container-title").map(|title| escape(&title)));
    let date = item.date_parts();
    push("year", date.first().map(i64::to_string));
    push("month", date.get(1).and_then(|month| MONTHS.get((*month as usize).wrapping_sub(1))).map(|month| month.to_string()));
    push("volume", item.text("volume").map(|value| escape(&value)));
    push("number", item.text("issue").or_else(|| item.text("number")).map(|value| escape(&value)));
    push("pages", item.text("page").map(|pages| escape(&pages.replace('-', "--").replace("----", "--"))));
    push(publisher_field(kind), item.text("publisher").map(|value| escape(&value)));
    push("address", item.text("publisher-place").map(|value| escape(&value)));
    push("isbn", item.text("ISBN"));
    push("issn", item.text("ISSN"));
    push("doi", item.text("DOI"));
    push("url", item.text("URL"));
    if let Some(arxiv_id) = item.arxiv_id() {
        push("eprint", Some(arxiv_id));
        push("archiveprefix", Some("arXiv".to_string()));
    }

    let body = fields
        .iter()
        .map(|(name, value)| format!("  {name} = {{{value}}}"))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("@{kind}{{{key},\n{body}\n}}\n")
}

pub fn existing_keys(text: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find('@') {
        rest = &rest[at + 1..];
        let Some(open) = rest.find(['{', '(']) else {
            break;
        };
        let entry_type = rest[..open].trim();
        if entry_type.is_empty() || !entry_type.chars().all(|character| character.is_ascii_alphabetic()) {
            continue;
        }
        let after_open = &rest[open + 1..];
        let key = after_open.split([',', '\n', '}', ')']).next().unwrap_or_default().trim();
        if !key.is_empty() && !entry_type.eq_ignore_ascii_case("comment") && !entry_type.eq_ignore_ascii_case("string") {
            keys.push(key.to_string());
        }
    }
    keys
}

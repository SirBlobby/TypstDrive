use super::csl::{CslItem, Person};

fn quote(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn yaml_key(key: &str) -> String {
    if key.chars().all(|character| character.is_ascii_alphanumeric() || character == '-' || character == '_') {
        key.to_string()
    } else {
        quote(key)
    }
}

fn date(parts: &[i64]) -> Option<String> {
    match parts {
        [year, month, day, ..] => Some(format!("{year:04}-{month:02}-{day:02}")),
        [year, month] => Some(format!("{year:04}-{month:02}")),
        [year] => Some(format!("{year:04}")),
        [] => None,
    }
}

struct Writer {
    lines: Vec<String>,
}

impl Writer {
    fn field(&mut self, indent: usize, name: &str, value: Option<String>) {
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            self.lines.push(format!("{}{name}: {}", " ".repeat(indent), quote(&value)));
        }
    }

    fn people(&mut self, indent: usize, name: &str, people: &[Person]) {
        if people.is_empty() {
            return;
        }
        self.lines.push(format!("{}{name}:", " ".repeat(indent)));
        for person in people {
            self.lines.push(format!("{}- {}", " ".repeat(indent + 2), quote(&person.display())));
        }
    }
}

fn types(item: &CslItem) -> (&'static str, Option<&'static str>) {
    match item.kind().as_str() {
        "article-journal" | "review" => ("article", Some("periodical")),
        "article-magazine" => ("article", Some("periodical")),
        "article-newspaper" => ("article", Some("newspaper")),
        "article" if item.text("container-title").is_some() => ("article", Some("periodical")),
        "book" => ("book", None),
        "chapter" => ("chapter", Some("book")),
        "entry-encyclopedia" | "entry-dictionary" => ("entry", Some("reference")),
        "paper-conference" => ("article", Some("proceedings")),
        "thesis" => ("thesis", None),
        "report" => ("report", None),
        "webpage" => ("web", None),
        "post-weblog" | "post" => ("post", None),
        "software" => ("repository", None),
        "manuscript" => ("manuscript", None),
        _ => ("misc", None),
    }
}

pub fn entry(key: &str, item: &CslItem) -> String {
    let (entry_type, parent_type) = types(item);
    let mut writer = Writer { lines: vec![format!("{}:", yaml_key(key)), format!("  type: {entry_type}")] };

    writer.field(2, "title", item.text("title"));
    writer.people(2, "author", &item.people("author"));
    writer.people(2, "editor", &item.people("editor"));
    writer.field(2, "date", date(&item.date_parts()));
    writer.field(2, "page-range", item.text("page"));

    let publisher_name = item.text("publisher");
    match (parent_type, item.text("container-title")) {
        (Some(parent_type), Some(container)) => {
            writer.lines.push("  parent:".to_string());
            writer.lines.push(format!("    type: {parent_type}"));
            writer.field(4, "title", Some(container));
            writer.field(4, "volume", item.text("volume"));
            writer.field(4, "issue", item.text("issue"));
            writer.field(4, "publisher", publisher_name);
        }
        _ => {
            writer.field(2, "volume", item.text("volume"));
            writer.field(2, "issue", item.text("issue"));
            match entry_type {
                "thesis" | "report" => writer.field(2, "organization", publisher_name),
                _ => writer.field(2, "publisher", publisher_name),
            }
        }
    }
    writer.field(2, "location", item.text("publisher-place"));
    writer.field(2, "url", item.text("URL"));

    let serial_numbers: Vec<(&str, Option<String>)> = vec![
        ("doi", item.text("DOI")),
        ("isbn", item.text("ISBN")),
        ("issn", item.text("ISSN")),
        ("arxiv", item.arxiv_id()),
    ];
    if serial_numbers.iter().any(|(_, value)| value.is_some()) {
        writer.lines.push("  serial-number:".to_string());
        for (name, value) in serial_numbers {
            writer.field(4, name, value);
        }
    }

    writer.lines.join("\n") + "\n"
}

pub fn existing_keys(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| !line.starts_with([' ', '\t', '#', '-']) && line.trim_end().ends_with(':'))
        .map(|line| line.trim_end().trim_end_matches(':').trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|key| !key.is_empty())
        .collect()
}

use std::collections::HashMap;

use axum::http::StatusCode;

use crate::compile::compiler::ProjectInput;
use crate::models::Document;

pub const TYPST_LANGUAGE: &str = "typst";
pub const LATEX_LANGUAGE: &str = "latex";

pub const TYPST_STARTER: &str = "== New Document";
pub const LATEX_STARTER: &str = "\\documentclass{article}\n\\title{New Document}\n\\author{}\n\n\\begin{document}\n\\maketitle\n\n\\section{Introduction}\nStart writing here.\n\n\\end{document}\n";

const MITEX_IMPORT: &str = "#import \"@preview/mitex:0.2.7\": *\n";
const ARTICLE_STYLE: &str = "#set page(paper: \"us-letter\", margin: 1in)\n\
#set text(font: \"New Computer Modern\", size: 11pt)\n\
#set par(justify: true, first-line-indent: 1.5em)\n\
#set heading(numbering: \"1.1\")\n\
#set math.equation(numbering: \"(1)\")\n\
#show heading: set block(above: 1.4em, below: 1em)\n";
const TODAY: &str = "datetime.today().display(\"[month repr:long] [day padding:none], [year]\")";

pub fn parse_language(requested: Option<&str>) -> Result<&'static str, (StatusCode, String)> {
    match requested.unwrap_or(TYPST_LANGUAGE) {
        TYPST_LANGUAGE => Ok(TYPST_LANGUAGE),
        LATEX_LANGUAGE => Ok(LATEX_LANGUAGE),
        other => Err((StatusCode::BAD_REQUEST, format!("Unsupported document language '{other}'"))),
    }
}

pub const LATEX_PROJECT_ENTRYPOINT: &str = "main.tex";
pub const TYPST_PROJECT_ENTRYPOINT: &str = "main.typ";

pub fn project_entrypoint(language: &str) -> &'static str {
    if language == LATEX_LANGUAGE { LATEX_PROJECT_ENTRYPOINT } else { TYPST_PROJECT_ENTRYPOINT }
}

pub fn starter_text(language: &str) -> &'static str {
    if language == LATEX_LANGUAGE { LATEX_STARTER } else { TYPST_STARTER }
}

pub fn is_latex(document: &Document) -> bool {
    document.language.as_deref() == Some(LATEX_LANGUAGE)
}

pub fn is_latex_path(path: &str) -> bool {
    path.to_lowercase().ends_with(".tex")
}

pub fn file_extension(language: Option<&str>) -> &'static str {
    if language == Some(LATEX_LANGUAGE) { "tex" } else { "typ" }
}

pub fn document_input(document: &Document, text: String, files: HashMap<String, Vec<u8>>) -> ProjectInput {
    if is_latex(document) {
        ProjectInput::single_named("main.tex", text, files)
    } else {
        ProjectInput::single(text, files)
    }
}

pub fn with_generated_entrypoint(mut input: ProjectInput) -> ProjectInput {
    if !is_latex_path(&input.entrypoint) {
        return input;
    }
    let latex_source = input
        .files
        .get(&input.entrypoint)
        .map(|bytes| String::from_utf8_lossy(bytes).to_string())
        .unwrap_or_default();
    let generated_path = generated_entrypoint_for(&input.entrypoint);
    input.files.insert(generated_path.clone(), latex_to_typst(&latex_source).into_bytes());
    input.entrypoint = generated_path;
    input
}

fn generated_entrypoint_for(latex_path: &str) -> String {
    let stem = &latex_path[..latex_path.len() - ".tex".len()];
    format!("{stem}.latex-generated.typ")
}

pub fn latex_to_typst(source: &str) -> String {
    let (preamble, body) = split_document(source);
    let mut typst = format!("{MITEX_IMPORT}{ARTICLE_STYLE}\n");

    match body.split_once("\\maketitle") {
        Some((before_title, after_title)) => {
            push_text_block(&mut typst, before_title);
            typst.push_str(&title_block(&strip_comments(preamble)));
            push_text_block(&mut typst, after_title);
        }
        None => push_text_block(&mut typst, body),
    }
    typst
}

fn split_document(source: &str) -> (&str, &str) {
    const BEGIN: &str = "\\begin{document}";
    const END: &str = "\\end{document}";
    let Some(begin_index) = source.find(BEGIN) else {
        return ("", source);
    };
    let body_start = begin_index + BEGIN.len();
    let body_end = source[body_start..].find(END).map(|offset| body_start + offset).unwrap_or(source.len());
    (&source[..begin_index], &source[body_start..body_end])
}

fn push_text_block(typst: &mut String, latex: &str) {
    if latex.trim().is_empty() {
        return;
    }
    typst.push_str(&format!("#mitext({})\n", typst_string(latex)));
}

fn title_block(preamble: &str) -> String {
    let title = command_argument(preamble, "title");
    let author = command_argument(preamble, "author").map(|author| author.replace("\\and", ", "));
    let date = match command_argument(preamble, "date") {
        Some(date) if date.trim() == "\\today" => Some(TODAY.to_string()),
        Some(date) if date.trim().is_empty() => None,
        Some(date) => Some(format!("mitext({})", typst_string(&date))),
        None => Some(TODAY.to_string()),
    };

    let mut lines = vec!["#align(center)[".to_string()];
    if let Some(title) = title.filter(|title| !title.trim().is_empty()) {
        lines.push(format!("  #block(text(size: 1.7em, mitext({})))", typst_string(&title)));
    }
    if let Some(author) = author.filter(|author| !author.trim().is_empty()) {
        lines.push(format!("  #block(text(size: 1.2em, mitext({})))", typst_string(&author)));
    }
    if let Some(date) = date {
        lines.push(format!("  #block(text(size: 1.2em, {date}))"));
    }
    lines.push("]\n#v(1.5em)\n".to_string());
    lines.join("\n")
}

fn command_argument(latex: &str, command: &str) -> Option<String> {
    let pattern = format!("\\{command}");
    let mut search_from = 0;
    while let Some(offset) = latex[search_from..].find(&pattern) {
        let after_command = search_from + offset + pattern.len();
        let rest = &latex[after_command..];
        let next_char = rest.chars().next();
        if matches!(next_char, Some('{') | Some('[')) || next_char.is_some_and(char::is_whitespace) {
            if let Some(argument) = braced_argument(rest) {
                return Some(argument);
            }
        }
        search_from = after_command;
    }
    None
}

fn braced_argument(text: &str) -> Option<String> {
    let mut remaining = text.trim_start();
    if remaining.starts_with('[') {
        remaining = remaining[remaining.find(']')? + 1..].trim_start();
    }
    if !remaining.starts_with('{') {
        return None;
    }

    let mut depth = 0;
    let mut previous_was_backslash = false;
    for (index, character) in remaining.char_indices() {
        if previous_was_backslash {
            previous_was_backslash = false;
            continue;
        }
        match character {
            '\\' => previous_was_backslash = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(remaining[1..index].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

fn strip_comments(latex: &str) -> String {
    latex
        .lines()
        .map(|line| {
            let mut previous_was_backslash = false;
            for (index, character) in line.char_indices() {
                if character == '%' && !previous_was_backslash {
                    return &line[..index];
                }
                previous_was_backslash = character == '\\' && !previous_was_backslash;
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn typst_string(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len() + 2);
    escaped.push('"');
    for character in text.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            other => escaped.push(other),
        }
    }
    escaped.push('"');
    escaped
}

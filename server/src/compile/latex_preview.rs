use std::collections::HashMap;

use crate::compile::compiler::{DocumentStats, ProjectInput};
use crate::compile::latex::is_latex_path;

const PAGES_FILE: &str = "latex-pages.pdf";
const BODY_START: &str = "\\begin{document}";
const BODY_END: &str = "\\end{document}";
const SPACED_SYMBOLS: &[char] = &['{', '}', '[', ']', '$', '&', '~', '^', '_', '#'];

pub enum PageChoice {
    All,
    One(usize),
}

fn read_pdf(pdf: Vec<u8>) -> Result<hayro_syntax::Pdf, String> {
    hayro_syntax::Pdf::new(pdf).map_err(|_| "The LaTeX output is not a readable PDF.".to_string())
}

pub fn pdf_page_count(pdf: &[u8]) -> Result<usize, String> {
    Ok(read_pdf(pdf.to_vec())?.pages().len())
}

fn page_markup(page_number: usize, width: f32, height: f32, scale: f32) -> String {
    let (width, height) = (width * scale, height * scale);
    format!(
        "#page(width: {width}pt, height: {height}pt, margin: 0pt)[#image(\"{PAGES_FILE}\", page: {page_number}, width: 100%, height: 100%, fit: \"stretch\")]\n"
    )
}

pub fn chosen_page_input(pdf: Vec<u8>, choice: PageChoice, scale: f32) -> Result<ProjectInput, String> {
    let document = read_pdf(pdf.clone())?;
    let pages = document.pages();
    if pages.is_empty() {
        return Err("The LaTeX document has no pages.".to_string());
    }
    let numbers: Vec<usize> = match choice {
        PageChoice::All => (1..=pages.len()).collect(),
        PageChoice::One(number) if number >= 1 && number <= pages.len() => vec![number],
        PageChoice::One(number) => {
            return Err(format!("Page {number} does not exist. The document has {} page(s).", pages.len()));
        }
    };
    let source: String = numbers
        .into_iter()
        .map(|number| {
            let (width, height) = pages[number - 1].render_dimensions();
            page_markup(number, width, height, scale)
        })
        .collect();
    Ok(ProjectInput::single(source, HashMap::from([(PAGES_FILE.to_string(), pdf)])))
}

fn without_comments(latex: &str) -> String {
    latex
        .lines()
        .map(|line| {
            let mut escaped = false;
            for (index, character) in line.char_indices() {
                if character == '%' && !escaped {
                    return &line[..index];
                }
                escaped = character == '\\' && !escaped;
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn document_body(latex: &str) -> &str {
    let Some(start) = latex.find(BODY_START) else { return latex };
    let body = &latex[start + BODY_START.len()..];
    body.find(BODY_END).map(|end| &body[..end]).unwrap_or(body)
}

fn readable_text(latex: &str) -> String {
    let mut text = String::with_capacity(latex.len());
    let mut characters = latex.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\\' {
            let mut skipped_name = false;
            while characters.peek().is_some_and(|next| next.is_ascii_alphabetic()) {
                characters.next();
                skipped_name = true;
            }
            if !skipped_name {
                characters.next();
            }
            text.push(' ');
        } else if SPACED_SYMBOLS.contains(&character) {
            text.push(' ');
        } else {
            text.push(character);
        }
    }
    text
}

pub fn source_stats(entrypoint: &str, files: &HashMap<String, Vec<u8>>, pages: usize) -> DocumentStats {
    let mut text = String::new();
    for (path, bytes) in files {
        if !is_latex_path(path) {
            continue;
        }
        let source = without_comments(&String::from_utf8_lossy(bytes));
        let content = if path == entrypoint { document_body(&source).to_string() } else { source };
        text.push_str(&readable_text(&content));
        text.push('\n');
    }
    DocumentStats {
        pages,
        words: text.split_whitespace().count(),
        characters: text.split_whitespace().collect::<Vec<_>>().join(" ").chars().count(),
        characters_excluding_spaces: text.chars().filter(|character| !character.is_whitespace()).count(),
    }
}

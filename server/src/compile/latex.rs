use std::collections::HashMap;

use axum::http::StatusCode;

use crate::compile::compiler::ProjectInput;
use crate::models::Document;

pub const TYPST_LANGUAGE: &str = "typst";
pub const LATEX_LANGUAGE: &str = "latex";

pub const TYPST_STARTER: &str = "== New Document";
pub const LATEX_STARTER: &str = "\\documentclass{article}\n\\title{New Document}\n\\author{}\n\n\\begin{document}\n\\maketitle\n\n\\section{Introduction}\nStart writing here.\n\n\\end{document}\n";

pub fn parse_language(requested: Option<&str>) -> Result<&'static str, (StatusCode, String)> {
    match requested.unwrap_or(TYPST_LANGUAGE) {
        TYPST_LANGUAGE => Ok(TYPST_LANGUAGE),
        LATEX_LANGUAGE => crate::compile::latex_policy::check_enabled().map(|_| LATEX_LANGUAGE),
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

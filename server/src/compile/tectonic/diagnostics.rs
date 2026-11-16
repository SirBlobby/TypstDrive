use std::path::Path;

use super::job::safe_relative_path;

const ERROR_PREFIX: &str = "error: ";
const GENERIC_MESSAGES: &[&str] = &[
    "something bad happened inside",
    "the XeTeX engine had an unrecoverable error",
    "halted on potentially-recoverable error",
    "the BibTeX engine had an unrecoverable error",
];
const SOURCE_EXTENSIONS: &[&str] = &[".tex", ".sty", ".cls", ".bib", ".ltx", ".bbl", ".aux"];

#[derive(Clone, Debug)]
pub struct LatexError {
    pub message: String,
    pub file: Option<String>,
    pub line: Option<usize>,
}

impl LatexError {
    pub fn general(message: impl Into<String>) -> Self {
        LatexError { message: message.into(), file: None, line: None }
    }
}

fn is_generic(message: &str) -> bool {
    GENERIC_MESSAGES.iter().any(|generic| message.starts_with(generic))
}

fn project_path(entry_folder: &Path, reported: &str) -> Option<String> {
    let relative = safe_relative_path(reported)?;
    let joined = safe_relative_path(&entry_folder.join(relative).to_string_lossy())?;
    Some(joined.to_string_lossy().into_owned())
}

fn located_error(entry_folder: &Path, text: &str) -> Option<LatexError> {
    let mut parts = text.splitn(3, ':');
    let file = parts.next()?;
    let line = parts.next()?.trim().parse::<usize>().ok()?;
    let message = parts.next()?.trim();
    if !SOURCE_EXTENSIONS.iter().any(|extension| file.to_lowercase().ends_with(extension)) {
        return None;
    }
    Some(LatexError {
        message: message.trim_start_matches('!').trim().to_string(),
        file: project_path(entry_folder, file),
        line: Some(line),
    })
}

pub fn parse_errors(output: &str, entrypoint: &str) -> Vec<LatexError> {
    let entry_folder = Path::new(entrypoint).parent().unwrap_or(Path::new(""));
    let mut errors: Vec<LatexError> = Vec::new();
    for line in output.lines() {
        let Some(text) = line.strip_prefix(ERROR_PREFIX).map(str::trim) else { continue };
        if text.is_empty() || is_generic(text) {
            continue;
        }
        let error = located_error(entry_folder, text).unwrap_or_else(|| LatexError::general(text));
        if !errors.iter().any(|known| known.message == error.message && known.line == error.line) {
            errors.push(error);
        }
    }
    if errors.is_empty() {
        errors.push(LatexError::general("LaTeX could not compile this document."));
    }
    errors
}

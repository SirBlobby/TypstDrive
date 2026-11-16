use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::compile::compiler::CompileFailure;
use crate::compile::render::{LatexProblem, RenderFailure};

#[derive(Serialize)]
pub struct CompileErrorDetail {
    message: String,
    severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<String>,
    line: Option<usize>,
    column: Option<usize>,
}

impl CompileErrorDetail {
    fn located(&self) -> String {
        let place = match (&self.file, self.line, self.column) {
            (Some(file), Some(line), _) => format!(" ({file}, line {line})"),
            (None, Some(line), Some(column)) => format!(" (line {line}, column {column})"),
            (None, Some(line), None) => format!(" (line {line})"),
            (Some(file), None, _) => format!(" ({file})"),
            (None, None, _) => String::new(),
        };
        format!("{}{place}", self.message)
    }
}

#[derive(Serialize)]
pub struct CompileErrorResponse {
    pub error: String,
    pub details: Vec<CompileErrorDetail>,
}

impl CompileErrorResponse {
    fn new(engine: &str, details: Vec<CompileErrorDetail>) -> Self {
        let summary = details.iter().map(CompileErrorDetail::located).collect::<Vec<_>>().join("; ");
        let error = if summary.is_empty() {
            format!("{engine} compilation failed.")
        } else {
            format!("{engine} compilation failed: {summary}")
        };
        CompileErrorResponse { error, details }
    }

    pub fn without_wrapper_lines(self, lines_before_code: usize) -> Self {
        if lines_before_code == 0 {
            return self;
        }
        let engine = self.error.split(' ').next().unwrap_or("Document").to_string();
        let details = self
            .details
            .into_iter()
            .map(|mut detail| {
                if detail.file.is_none() {
                    detail.line = detail.line.and_then(|line| line.checked_sub(lines_before_code)).filter(|line| *line > 0);
                    if detail.line.is_none() {
                        detail.column = None;
                    }
                }
                detail
            })
            .collect();
        CompileErrorResponse::new(&engine, details)
    }
}

fn line_and_column(code: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    for (index, character) in code.char_indices() {
        if index >= offset {
            break;
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

pub fn typst_error_report(source: Option<&str>, diagnostics: CompileFailure) -> CompileErrorResponse {
    let details = diagnostics
        .into_iter()
        .map(|(diagnostic, range)| {
            let position = match (source, range) {
                (Some(code), Some(range)) => Some(line_and_column(code, range.start)),
                _ => None,
            };
            CompileErrorDetail {
                message: diagnostic.message.to_string(),
                severity: format!("{:?}", diagnostic.severity).to_lowercase(),
                file: None,
                line: position.map(|(line, _)| line),
                column: position.map(|(_, column)| column),
            }
        })
        .collect();
    CompileErrorResponse::new("Typst", details)
}

pub fn latex_error_report(entrypoint: &str, problems: Vec<LatexProblem>) -> CompileErrorResponse {
    let details = problems
        .into_iter()
        .map(|problem| CompileErrorDetail {
            message: problem.message,
            severity: "error".to_string(),
            file: problem.file.filter(|file| file != entrypoint),
            line: problem.line,
            column: None,
        })
        .collect();
    CompileErrorResponse::new("LaTeX", details)
}

pub enum FailureReport {
    Compile(CompileErrorResponse),
    Other(Response),
}

pub fn failure_report(source: Option<&str>, failure: RenderFailure) -> FailureReport {
    match failure {
        RenderFailure::Typst(diagnostics) => FailureReport::Compile(typst_error_report(source, diagnostics)),
        RenderFailure::Latex { entrypoint, problems } => FailureReport::Compile(latex_error_report(&entrypoint, problems)),
        RenderFailure::Unsupported(message) => FailureReport::Compile(CompileErrorResponse { error: message, details: Vec::new() }),
        RenderFailure::LatexTurnedOff => {
            FailureReport::Other((StatusCode::FORBIDDEN, crate::compile::latex_policy::TURNED_OFF_MESSAGE).into_response())
        }
        RenderFailure::Stopped => {
            FailureReport::Other((StatusCode::INTERNAL_SERVER_ERROR, "Compilation stopped unexpectedly").into_response())
        }
    }
}

pub fn failure_response(source: Option<&str>, lines_before_code: usize, failure: RenderFailure) -> Response {
    match failure_report(source, failure) {
        FailureReport::Compile(report) => {
            (StatusCode::UNPROCESSABLE_ENTITY, Json(report.without_wrapper_lines(lines_before_code))).into_response()
        }
        FailureReport::Other(response) => response,
    }
}

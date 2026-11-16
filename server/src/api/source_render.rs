use std::collections::HashSet;

use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::api::authentication::ApiCaller;
use crate::api::errors::{failure_report, failure_response, CompileErrorDetail, FailureReport};
use crate::api::options::{content_response, RenderFormat};
use crate::api::render::InlineFile;
use crate::api::render_cache::{add_inline_files, cache_key, cached_render, prune_cache, store_render};
use crate::compile::compiler::ProjectInput;
use crate::compile::render::{self, PageImage, RenderFailure};
use crate::drive::file_data::account_files;
use crate::AppState;

#[derive(Clone, Copy, PartialEq)]
pub enum Language {
    Typst,
    Latex,
}

impl Language {
    fn entrypoint(self) -> &'static str {
        match self {
            Language::Typst => "main.typ",
            Language::Latex => "main.tex",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Language::Typst => "typst",
            Language::Latex => "latex",
        }
    }
}

pub struct SourceJob {
    pub language: Language,
    pub kind: &'static str,
    pub text: String,
    pub lines_before_code: usize,
    pub files: Option<Vec<InlineFile>>,
}

impl SourceJob {
    pub fn document(language: Language, text: String, files: Option<Vec<InlineFile>>) -> Self {
        SourceJob { language, kind: "document", text, lines_before_code: 0, files }
    }

    fn typst_source(&self) -> Option<&str> {
        (self.language == Language::Typst).then_some(self.text.as_str())
    }

    fn cache_label(&self, format: RenderFormat) -> String {
        format!("{}-{}-{}", self.language.label(), self.kind, format.as_str())
    }
}

pub async fn project_input(state: &AppState, caller: &ApiCaller, job: &SourceJob) -> ProjectInput {
    let mut files = account_files(state, &caller.account_id, &[job.text.as_str()], &HashSet::new()).await;
    add_inline_files(&mut files, &job.files);
    ProjectInput::single_named(job.language.entrypoint(), job.text.clone(), files)
}

pub async fn render_output(input: ProjectInput, format: RenderFormat, image: PageImage) -> Result<Vec<u8>, RenderFailure> {
    match format {
        RenderFormat::Pdf => render::pdf(input).await,
        RenderFormat::Png => render::png_page(input, image).await,
        RenderFormat::Svg => render::svg_page(input, image).await.map(String::into_bytes),
        RenderFormat::Html => render::html(input).await.map(String::into_bytes),
    }
}

pub async fn render_job(state: &AppState, caller: &ApiCaller, job: SourceJob, format: RenderFormat, image: PageImage) -> Response {
    prune_cache(state).await;
    let label = job.cache_label(format);
    let page = image.page.to_string();
    let scale = image.scale.to_string();
    let key = cache_key(&caller.account_id, &[&label, &page, &scale, &job.text], &job.files);
    if let Some(data) = cached_render(state, &key, &label).await {
        return content_response(format.content_type(), data);
    }

    let input = project_input(state, caller, &job).await;
    match render_output(input, format, image).await {
        Ok(data) => {
            store_render(state, &key, &label, &data).await;
            content_response(format.content_type(), data)
        }
        Err(failure) => failure_response(job.typst_source(), job.lines_before_code, failure),
    }
}

#[derive(Serialize)]
struct CheckResponse {
    valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pages: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    details: Vec<CompileErrorDetail>,
}

pub async fn check_job(state: &AppState, caller: &ApiCaller, job: SourceJob) -> Response {
    let input = project_input(state, caller, &job).await;
    match render::page_count(input).await {
        Ok(pages) => Json(CheckResponse { valid: true, pages: Some(pages), error: None, details: Vec::new() }).into_response(),
        Err(failure) => match failure_report(job.typst_source(), failure) {
            FailureReport::Compile(report) => {
                let report = report.without_wrapper_lines(job.lines_before_code);
                Json(CheckResponse { valid: false, pages: None, error: Some(report.error), details: report.details }).into_response()
            }
            FailureReport::Other(response) => response,
        },
    }
}

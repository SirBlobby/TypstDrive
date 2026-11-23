use std::collections::HashMap;
use std::time::Instant;

use crate::compile::compiler::{run_blocking, ClickTarget, CompileFailure, PageOutput, PagePoint, ProjectInput, TypstCompiler};
use crate::compile::latex::is_latex_path;
use crate::compile::latex_policy;
use crate::compile::latex_preview::{chosen_page_input, pdf_page_count, source_stats, PageChoice};
use crate::compile::tectonic::{compile_to_pdf, LatexError};
use crate::metrics::{self, Metric};

pub use crate::compile::compiler::Preview;

pub struct LatexProblem {
    pub message: String,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub range: Option<(usize, usize)>,
}

impl LatexProblem {
    pub fn located_message(&self, entrypoint: &str) -> String {
        match (&self.file, self.line) {
            (Some(file), Some(line)) if file != entrypoint => format!("{file}, line {line}: {}", self.message),
            (_, Some(line)) => format!("Line {line}: {}", self.message),
            _ => self.message.clone(),
        }
    }
}

pub enum RenderFailure {
    Typst(CompileFailure),
    Latex { entrypoint: String, problems: Vec<LatexProblem> },
    Unsupported(String),
    LatexTurnedOff,
    Stopped,
}

fn line_range(text: &str, line_number: usize) -> Option<(usize, usize)> {
    let mut offset = 0;
    for (index, line) in text.split('\n').enumerate() {
        let length = line.trim_end_matches('\r').encode_utf16().count();
        if index + 1 == line_number {
            return Some((offset, offset + length));
        }
        offset += line.encode_utf16().count() + 1;
    }
    None
}

fn locate(errors: Vec<LatexError>, files: &HashMap<String, Vec<u8>>) -> Vec<LatexProblem> {
    errors
        .into_iter()
        .map(|error| {
            let range = match (&error.file, error.line) {
                (Some(file), Some(line)) => files.get(file).and_then(|bytes| line_range(&String::from_utf8_lossy(bytes), line)),
                _ => None,
            };
            LatexProblem { message: error.message, file: error.file, line: error.line, range }
        })
        .collect()
}

pub fn is_latex(input: &ProjectInput) -> bool {
    is_latex_path(&input.entrypoint)
}

pub async fn latex_pdf(input: &ProjectInput) -> Result<Vec<u8>, RenderFailure> {
    if !latex_policy::is_enabled() {
        return Err(RenderFailure::LatexTurnedOff);
    }
    compile_to_pdf(&input.entrypoint, input.files.clone()).await.map_err(|errors| RenderFailure::Latex {
        entrypoint: input.entrypoint.clone(),
        problems: locate(errors, &input.files),
    })
}

fn general_latex_failure(input: &ProjectInput, message: String) -> RenderFailure {
    RenderFailure::Latex {
        entrypoint: input.entrypoint.clone(),
        problems: vec![LatexProblem { message, file: None, line: None, range: None }],
    }
}

async fn latex_pages(input: &ProjectInput) -> Result<ProjectInput, RenderFailure> {
    let pdf = latex_pdf(input).await?;
    chosen_page_input(pdf, PageChoice::All, 1.0).map_err(|message| general_latex_failure(input, message))
}

pub struct PageImage {
    pub page: usize,
    pub scale: f32,
}

impl Default for PageImage {
    fn default() -> Self {
        PageImage { page: 1, scale: 1.0 }
    }
}

fn missing_page(page: usize, pages: usize) -> RenderFailure {
    RenderFailure::Unsupported(format!("Page {page} does not exist. The document has {pages} page(s)."))
}

fn chosen_page<T>(output: PageOutput<T>, page: usize) -> Result<T, RenderFailure> {
    match output {
        PageOutput::Page(value) => Ok(value),
        PageOutput::Missing { pages } => Err(missing_page(page, pages)),
    }
}

pub async fn png_page(input: ProjectInput, image: PageImage) -> Result<Vec<u8>, RenderFailure> {
    if is_latex(&input) {
        return latex_png(input, image).await;
    }
    let PageImage { page, scale } = image;
    let output = typst_job(input, move |compiler, input| compiler.export_png_page(input, page, scale)).await?;
    chosen_page(output, page)
}

pub async fn svg_page(input: ProjectInput, image: PageImage) -> Result<String, RenderFailure> {
    if is_latex(&input) {
        return latex_svg(input, image).await;
    }
    let page = image.page;
    let output = typst_job(input, move |compiler, input| compiler.export_svg_page(input, page)).await?;
    chosen_page(output, page)
}

pub async fn page_count(input: ProjectInput) -> Result<usize, RenderFailure> {
    if is_latex(&input) {
        return latex_page_count(input).await;
    }
    typst_job(input, |compiler, input| compiler.page_count(input)).await
}

async fn latex_image_input(input: &ProjectInput, image: &PageImage) -> Result<ProjectInput, RenderFailure> {
    let pdf = latex_pdf(input).await?;
    chosen_page_input(pdf, PageChoice::One(image.page), image.scale).map_err(RenderFailure::Unsupported)
}

async fn latex_png(input: ProjectInput, image: PageImage) -> Result<Vec<u8>, RenderFailure> {
    let pages = latex_image_input(&input, &image).await?;
    latex_page_job(pages, |compiler, pages| compiler.export_png(pages)).await
}

async fn latex_svg(input: ProjectInput, image: PageImage) -> Result<String, RenderFailure> {
    let pages = latex_image_input(&input, &image).await?;
    latex_page_job(pages, |compiler, pages| compiler.compile_svg(pages).map(|preview| preview.thumbnail)).await
}

async fn latex_page_count(input: ProjectInput) -> Result<usize, RenderFailure> {
    let pdf = latex_pdf(&input).await?;
    pdf_page_count(&pdf).map_err(|message| general_latex_failure(&input, message))
}

type TypstWork<T> = Box<dyn FnOnce(&TypstCompiler, ProjectInput) -> Result<T, CompileFailure> + Send + 'static>;

async fn run_typst<T: Send + 'static>(input: ProjectInput, job: TypstWork<T>, timed: bool) -> Result<T, RenderFailure> {
    let outcome = run_blocking(move |compiler| {
        let started = Instant::now();
        let result = job(compiler, input);
        if timed {
            metrics::record(Metric::TypstCompile, started.elapsed(), result.is_err());
        }
        result
    })
    .await;
    match outcome {
        Some(result) => result.map_err(RenderFailure::Typst),
        None => Err(RenderFailure::Stopped),
    }
}

async fn typst_job<T: Send + 'static>(
    input: ProjectInput,
    job: impl FnOnce(&TypstCompiler, ProjectInput) -> Result<T, CompileFailure> + Send + 'static,
) -> Result<T, RenderFailure> {
    run_typst(input, Box::new(job), true).await
}

async fn latex_page_job<T: Send + 'static>(
    pages: ProjectInput,
    job: impl FnOnce(&TypstCompiler, ProjectInput) -> Result<T, CompileFailure> + Send + 'static,
) -> Result<T, RenderFailure> {
    run_typst(pages, Box::new(job), false).await
}

pub async fn preview(input: ProjectInput) -> Result<Preview, RenderFailure> {
    if !is_latex(&input) {
        return typst_job(input, |compiler, input| compiler.compile_svg(input)).await;
    }
    let pages = latex_pages(&input).await?;
    let preview = latex_page_job(pages, |compiler, pages| compiler.compile_svg(pages)).await?;
    let stats = source_stats(&input.entrypoint, &input.files, preview.svgs.len());
    Ok(Preview { stats, notes: None, outside_dependencies: None, ..preview })
}

pub async fn pdf(input: ProjectInput) -> Result<Vec<u8>, RenderFailure> {
    if is_latex(&input) {
        return latex_pdf(&input).await;
    }
    typst_job(input, |compiler, input| compiler.export_pdf(input)).await
}

pub async fn png(input: ProjectInput) -> Result<Vec<u8>, RenderFailure> {
    let job = |compiler: &TypstCompiler, input| compiler.export_png(input);
    if is_latex(&input) {
        return latex_page_job(latex_pages(&input).await?, job).await;
    }
    typst_job(input, job).await
}

pub async fn svg(input: ProjectInput) -> Result<String, RenderFailure> {
    let job = |compiler: &TypstCompiler, input| compiler.compile_svg(input).map(|preview| preview.svgs.join("\n"));
    if is_latex(&input) {
        return latex_page_job(latex_pages(&input).await?, job).await;
    }
    typst_job(input, job).await
}

pub async fn html(input: ProjectInput) -> Result<String, RenderFailure> {
    if is_latex(&input) {
        return Err(RenderFailure::Unsupported("HTML export is not available for LaTeX documents.".to_string()));
    }
    typst_job(input, |compiler, input| compiler.export_html(input)).await
}

pub async fn cursor_positions(input: ProjectInput, path: String, offset: usize) -> Vec<PagePoint> {
    if is_latex(&input) {
        return Vec::new();
    }
    run_blocking(move |compiler| compiler.cursor_positions(input, &path, offset)).await.unwrap_or_default()
}

pub async fn click_target(input: ProjectInput, page: usize, x: f64, y: f64) -> Option<ClickTarget> {
    if is_latex(&input) {
        return None;
    }
    run_blocking(move |compiler| compiler.click_target(input, page, x, y)).await.flatten()
}

use crate::compile::world::{MemoryWorld, UsedResources};
use serde::Serialize;
use std::collections::HashMap;
use typst::diag::{SourceDiagnostic, Warned};
use typst::format::Complete;
use typst::layout::{Frame, FrameItem};
use typst::utils::Scalar;
use typst::model::Document;
use typst_html::{HtmlDocument, HtmlOptions};
use typst_layout::PagedDocument;
use typst_pdf::{pdf, PdfOptions};
use typst_render::{render, PngFormat, RenderOptions};
use typst_svg::{SvgFormat, SvgOptions};

/// Rendered pages for the live preview.
pub struct Preview {
    pub svgs: Vec<String>,
    pub thumbnail: String,
    pub stats: DocumentStats,
    pub notes: Option<Vec<Option<String>>>,
}

#[derive(Serialize, Clone)]
pub struct DocumentStats {
    pub pages: usize,
    pub words: usize,
    pub characters: usize,
    pub characters_excluding_spaces: usize,
}

fn extract_stats(doc: &PagedDocument) -> DocumentStats {
    let mut text = String::new();
    let pages = doc.pages().len();
    for page in doc.pages() {
        extract_frame_text(&page.frame, &mut text);
    }

    let words = text.split_whitespace().count();
    let characters = text.chars().count();
    let characters_excluding_spaces = text.chars().filter(|c| !c.is_whitespace()).count();

    DocumentStats {
        pages,
        words,
        characters,
        characters_excluding_spaces,
    }
}

fn extract_frame_text(frame: &Frame, text: &mut String) {
    for (_, item) in frame.items() {
        match item {
            FrameItem::Text(text_item) => {
                text.push_str(&text_item.text);
                text.push(' ');
            }
            FrameItem::Group(group) => {
                extract_frame_text(&group.frame, text);
            }
            _ => {}
        }
    }
}

fn svg_options(doc: &PagedDocument) -> SvgOptions<Complete> {
    let requested: SvgOptions = SvgOptions::default();
    requested.resolve(doc.options().get::<SvgFormat>())
}

fn png_options(doc: &PagedDocument) -> RenderOptions<Complete> {
    let requested: RenderOptions = RenderOptions::default();
    requested.resolve(doc.options().get::<PngFormat>())
}

pub struct ProjectInput {
    pub entrypoint: String,
    pub files: HashMap<String, Vec<u8>>,
    pub packages: HashMap<String, HashMap<String, Vec<u8>>>,
}

impl ProjectInput {
    pub fn single(text: String, files: HashMap<String, Vec<u8>>) -> Self {
        let mut project_files = files;
        project_files.insert("main.typ".to_string(), text.into_bytes());
        Self {
            entrypoint: "main.typ".to_string(),
            files: project_files,
            packages: HashMap::new(),
        }
    }

    pub fn single_named(entrypoint: &str, text: String, files: HashMap<String, Vec<u8>>) -> Self {
        let mut project_files = files;
        project_files.insert(entrypoint.to_string(), text.into_bytes());
        Self {
            entrypoint: entrypoint.to_string(),
            files: project_files,
            packages: HashMap::new(),
        }
    }

    fn into_world(self, enable_html: bool) -> MemoryWorld {
        MemoryWorld::new_project(self.entrypoint, self.files, self.packages, enable_html)
    }
}

pub type CompileFailure = Vec<(SourceDiagnostic, Option<std::ops::Range<usize>>)>;

const MEMO_CACHE_MAX_AGE: usize = 10;

pub struct TypstCompiler;

pub enum PageOutput<T> {
    Page(T),
    Missing { pages: usize },
}

fn paged_document(input: ProjectInput) -> Result<PagedDocument, CompileFailure> {
    let world = input.into_world(false);
    let Warned { output, warnings: _ } = typst::compile::<PagedDocument>(&world);
    output.map_err(|errors| {
        use typst::WorldExt;
        errors.into_iter().map(|diagnostic| {
            let range = world.range(diagnostic.span);
            (diagnostic, range)
        }).collect()
    })
}

pub async fn run_blocking<T, F>(job: F) -> Option<T>
where
    F: FnOnce(&TypstCompiler) -> T + Send + 'static,
    T: Send + 'static,
{
    let result = tokio::task::spawn_blocking(move || {
        let output = job(&TypstCompiler);
        typst::comemo::evict(MEMO_CACHE_MAX_AGE);
        output
    })
    .await;
    match result {
        Ok(output) => Some(output),
        Err(error) => {
            tracing::error!("Typst compilation task failed: {error}");
            None
        }
    }
}

impl TypstCompiler {
    pub fn used_resources(&self, input: ProjectInput) -> UsedResources {
        let world = input.into_world(false);
        let _ = typst::compile::<PagedDocument>(&world);
        world.used_resources()
    }

    pub fn compile_svg(
        &self,
        input: ProjectInput,
    ) -> Result<Preview, Vec<(SourceDiagnostic, Option<std::ops::Range<usize>>)>> {
        let world = input.into_world(false);
        match typst::compile::<PagedDocument>(&world) {
            Warned {
                output: Ok(doc),
                warnings: _,
            } => {
                let stats = extract_stats(&doc);
                let options = svg_options(&doc);
                let svgs = doc
                    .pages()
                    .iter()
                    .map(|page| typst_svg::svg(page, &options))
                    .collect();
                let thumbnail = if let Some(page) = doc.pages().first() {
                    typst_svg::svg(page, &options)
                } else {
                    String::new()
                };
                let notes = crate::compile::speaker_notes::page_notes(&doc);
                Ok(Preview { svgs, thumbnail, stats, notes })
            }
            Warned {
                output: Err(errors),
                warnings: _,
            } => {
                use typst::WorldExt;
                let diag = errors
                    .into_iter()
                    .map(|d| {
                        let range = world.range(d.span);
                        (d, range)
                    })
                    .collect();
                Err(diag)
            }
        }
    }

    pub fn export_png_page(&self, input: ProjectInput, page_number: usize, scale: f32) -> Result<PageOutput<Vec<u8>>, CompileFailure> {
        let document = paged_document(input)?;
        let Some(page) = page_number.checked_sub(1).and_then(|index| document.pages().get(index)) else {
            return Ok(PageOutput::Missing { pages: document.pages().len() });
        };
        let mut options = png_options(&document);
        options.format.ppi.v = Scalar::new(options.format.ppi.v.get() * scale as f64);
        Ok(PageOutput::Page(render(page, &options).encode_png().unwrap_or_default()))
    }

    pub fn export_svg_page(&self, input: ProjectInput, page_number: usize) -> Result<PageOutput<String>, CompileFailure> {
        let document = paged_document(input)?;
        let Some(page) = page_number.checked_sub(1).and_then(|index| document.pages().get(index)) else {
            return Ok(PageOutput::Missing { pages: document.pages().len() });
        };
        Ok(PageOutput::Page(typst_svg::svg(page, &svg_options(&document))))
    }

    pub fn page_count(&self, input: ProjectInput) -> Result<usize, CompileFailure> {
        paged_document(input).map(|document| document.pages().len())
    }

    pub fn export_pdf(
        &self,
        input: ProjectInput,
    ) -> Result<Vec<u8>, Vec<(SourceDiagnostic, Option<std::ops::Range<usize>>)>> {
        let world = input.into_world(false);
        match typst::compile::<PagedDocument>(&world) {
            Warned {
                output: Ok(doc),
                warnings: _,
            } => {
                let opts = PdfOptions::default();
                match pdf(&doc, &opts).output {
                    Ok(bytes) => Ok(bytes),
                    Err(_) => Err(vec![]),
                }
            }
            Warned {
                output: Err(errors),
                warnings: _,
            } => {
                use typst::WorldExt;
                Err(errors
                    .into_iter()
                    .map(|d| {
                        let range = world.range(d.span);
                        (d, range)
                    })
                    .collect())
            }
        }
    }

    pub fn export_png(
        &self,
        input: ProjectInput,
    ) -> Result<Vec<u8>, Vec<(SourceDiagnostic, Option<std::ops::Range<usize>>)>> {
        let world = input.into_world(false);
        match typst::compile::<PagedDocument>(&world) {
            Warned {
                output: Ok(doc),
                warnings: _,
            } => {
                if let Some(page) = doc.pages().first() {
                    let options = png_options(&doc);
                    let pixmap = render(page, &options);
                    if let Ok(encoded) = pixmap.encode_png() {
                        return Ok(encoded);
                    }
                }
                Ok(vec![])
            }
            Warned {
                output: Err(errors),
                warnings: _,
            } => {
                use typst::WorldExt;
                Err(errors
                    .into_iter()
                    .map(|d| {
                        let range = world.range(d.span);
                        (d, range)
                    })
                    .collect())
            }
        }
    }

    pub fn export_html(
        &self,
        input: ProjectInput,
    ) -> Result<String, Vec<(SourceDiagnostic, Option<std::ops::Range<usize>>)>> {
        let world = input.into_world(true);
        let document = match typst::compile::<HtmlDocument>(&world) {
            Warned {
                output: Ok(document),
                warnings: _,
            } => document,
            Warned {
                output: Err(errors),
                warnings: _,
            } => {
                use typst::WorldExt;
                return Err(errors
                    .into_iter()
                    .map(|d| {
                        let range = world.range(d.span);
                        (d, range)
                    })
                    .collect());
            }
        };

        match typst_html::html(&document, &HtmlOptions::default()) {
            Ok(html) => Ok(html),
            Err(errors) => {
                use typst::WorldExt;
                Err(errors
                    .into_iter()
                    .map(|d| {
                        let range = world.range(d.span);
                        (d, range)
                    })
                    .collect())
            }
        }
    }
}

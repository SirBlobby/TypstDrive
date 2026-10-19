use crate::world::{MemoryWorld, UsedResources};
use serde::Serialize;
use std::collections::HashMap;
use typst::diag::{SourceDiagnostic, Warned};
use typst::format::Complete;
use typst::layout::{Frame, FrameItem};
use typst::model::Document;
use typst_html::{HtmlDocument, HtmlOptions};
use typst_layout::PagedDocument;
use typst_pdf::{pdf, PdfOptions};
use typst_render::{render, PngFormat, RenderOptions};
use typst_svg::{SvgFormat, SvgOptions};

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

    fn into_world(self, enable_html: bool) -> MemoryWorld {
        MemoryWorld::new_project(self.entrypoint, self.files, self.packages, enable_html)
    }
}

pub struct TypstCompiler;

impl TypstCompiler {
    pub fn new() -> Self {
        Self
    }

    pub fn used_resources(&self, input: ProjectInput) -> UsedResources {
        let world = input.into_world(false);
        let _ = typst::compile::<PagedDocument>(&world);
        world.used_resources()
    }

    pub fn compile_svg(
        &self,
        input: ProjectInput,
    ) -> Result<
        (Vec<String>, String, DocumentStats),
        Vec<(SourceDiagnostic, Option<std::ops::Range<usize>>)>,
    > {
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
                Ok((svgs, thumbnail, stats))
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

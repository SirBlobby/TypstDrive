use crate::world::MemoryWorld;
use serde::Serialize;
use std::collections::HashMap;
use typst::diag::{SourceDiagnostic, Warned};
use typst::layout::{Frame, FrameItem};
use typst_layout::PagedDocument;
use typst_pdf::{pdf, PdfOptions};
use typst_render::{render, RenderOptions};
use typst_svg::SvgOptions;

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

pub struct TypstCompiler;

impl TypstCompiler {
    pub fn new() -> Self {
        Self
    }

    pub fn compile_svg(
        &self,
        text: String,
        files: HashMap<String, Vec<u8>>,
    ) -> Result<
        (Vec<String>, String, DocumentStats),
        Vec<(SourceDiagnostic, Option<std::ops::Range<usize>>)>,
    > {
        let world = MemoryWorld::new(text, files);
        match typst::compile::<PagedDocument>(&world) {
            Warned {
                output: Ok(doc),
                warnings: _,
            } => {
                let stats = extract_stats(&doc);
                let options = SvgOptions::default();
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
        text: String,
        files: HashMap<String, Vec<u8>>,
    ) -> Result<Vec<u8>, Vec<(SourceDiagnostic, Option<std::ops::Range<usize>>)>> {
        let world = MemoryWorld::new(text, files);
        match typst::compile::<PagedDocument>(&world) {
            Warned {
                output: Ok(doc),
                warnings: _,
            } => {
                let opts = PdfOptions::default();
                match pdf(&doc, &opts) {
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
        text: String,
        files: HashMap<String, Vec<u8>>,
    ) -> Result<Vec<u8>, Vec<(SourceDiagnostic, Option<std::ops::Range<usize>>)>> {
        let world = MemoryWorld::new(text, files);
        match typst::compile::<PagedDocument>(&world) {
            Warned {
                output: Ok(doc),
                warnings: _,
            } => {
                if let Some(page) = doc.pages().first() {
                    let options = RenderOptions {
                        pixel_per_pt: 2.0,
                        ..RenderOptions::default()
                    };
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
}

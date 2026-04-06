use crate::world::MemoryWorld;
use std::collections::HashMap;
use typst::diag::{SourceDiagnostic, Warned};
use typst_layout::PagedDocument;
use typst_pdf::{pdf, PdfOptions};
use typst_render::render;

pub struct TypstCompiler;

impl TypstCompiler {
    pub fn new() -> Self {
        Self
    }

    pub fn compile_svg(
        &self,
        text: String,
        files: HashMap<String, Vec<u8>>,
    ) -> Result<(Vec<String>, String), Vec<(SourceDiagnostic, Option<std::ops::Range<usize>>)>> {
        let world = MemoryWorld::new(text, files);
        match typst::compile::<PagedDocument>(&world) {
            Warned {
                output: Ok(doc),
                warnings: _,
            } => {
                let svgs = doc.pages().iter().map(typst_svg::svg).collect();
                let thumbnail = if let Some(page) = doc.pages().first() {
                    typst_svg::svg(page)
                } else {
                    String::new()
                };
                Ok((svgs, thumbnail))
            }
            Warned {
                output: Err(errors),
                warnings: _,
            } => {
                use typst::World;
                let diag = errors.into_iter().map(|d| {
                    let range = d.span.id().and_then(|id| world.source(id).ok()).and_then(|s| s.range(d.span));
                    (d, range)
                }).collect();
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
                use typst::World;
                Err(errors.into_iter().map(|d| {
                    let range = d.span.id().and_then(|id| world.source(id).ok()).and_then(|s| s.range(d.span));
                    (d, range)
                }).collect())
            },
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
                    let pixmap = render(page, 2.0);
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
                use typst::World;
                Err(errors.into_iter().map(|d| {
                    let range = d.span.id().and_then(|id| world.source(id).ok()).and_then(|s| s.range(d.span));
                    (d, range)
                }).collect())
            },
        }
    }
}

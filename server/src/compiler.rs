use crate::world::MemoryWorld;
use std::collections::HashMap;
use typst::diag::{SourceDiagnostic, Warned};
use typst::layout::PagedDocument;
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
    ) -> Result<(Vec<String>, String), Vec<SourceDiagnostic>> {
        let world = MemoryWorld::new(text, files);
        match typst::compile::<PagedDocument>(&world) {
            Warned {
                output: Ok(doc),
                warnings: _,
            } => {
                let svgs = doc.pages.iter().map(|page| typst_svg::svg(page)).collect();
                let thumbnail = if let Some(page) = doc.pages.first() {
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
                let diag = errors.into_iter().collect();
                Err(diag)
            }
        }
    }

    pub fn export_pdf(
        &self,
        text: String,
        files: HashMap<String, Vec<u8>>,
    ) -> Result<Vec<u8>, Vec<SourceDiagnostic>> {
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
            } => Err(errors.into_iter().collect()),
        }
    }

    pub fn export_png(
        &self,
        text: String,
        files: HashMap<String, Vec<u8>>,
    ) -> Result<Vec<u8>, Vec<SourceDiagnostic>> {
        let world = MemoryWorld::new(text, files);
        match typst::compile::<PagedDocument>(&world) {
            Warned {
                output: Ok(doc),
                warnings: _,
            } => {
                if let Some(page) = doc.pages.first() {
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
            } => Err(errors.into_iter().collect()),
        }
    }
}

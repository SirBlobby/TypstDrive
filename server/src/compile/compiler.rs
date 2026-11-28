use crate::compile::speaker_notes::Note;
use crate::compile::world::{project_file_id, project_path, MemoryWorld, TodayAnswer, UsedResources};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use typst::diag::{SourceDiagnostic, Warned};
use typst::format::Complete;
use typst::introspection::PagedPosition;
use typst::layout::{Abs, Frame, FrameItem, Point};
use typst::utils::Scalar;
use typst::model::Document;
use typst::World;
use typst_ide::Jump;
use typst_html::{HtmlDocument, HtmlOptions};
use typst_layout::PagedDocument;
use typst_pdf::{pdf, PdfOptions};
use typst_render::{render, PngFormat, RenderOptions};
use typst_svg::{SvgFormat, SvgOptions};

pub struct Preview {
    pub svgs: Vec<String>,
    pub thumbnail: String,
    pub stats: DocumentStats,
    pub notes: Option<Vec<Option<Note>>>,
    pub outside_dependencies: Option<Vec<TodayAnswer>>,
}

#[derive(Serialize, Deserialize, Clone)]
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

pub(crate) fn svg_options(doc: &PagedDocument) -> SvgOptions<Complete> {
    let requested: SvgOptions = SvgOptions::default();
    requested.resolve(doc.options().get::<SvgFormat>())
}

fn export_svg_pages(pages: &[typst_layout::Page], options: &SvgOptions<Complete>) -> Vec<String> {
    let threads = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(8)
        .min(pages.len());
    if threads <= 1 {
        return pages.iter().map(|page| typst_svg::svg(page, options)).collect();
    }
    let mut svgs = vec![String::new(); pages.len()];
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|worker| {
                scope.spawn(move || {
                    (worker..pages.len())
                        .step_by(threads)
                        .map(|index| (index, typst_svg::svg(&pages[index], options)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        for worker in workers {
            for (index, svg) in worker.join().expect("SVG export thread panicked") {
                svgs[index] = svg;
            }
        }
    });
    svgs
}

fn png_options(doc: &PagedDocument) -> RenderOptions<Complete> {
    let requested: RenderOptions = RenderOptions::default();
    requested.resolve(doc.options().get::<PngFormat>())
}

#[derive(Clone)]
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

#[derive(Serialize)]
pub struct PagePoint {
    pub page: usize,
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ClickTarget {
    Source { path: String, offset: usize },
    Page(PagePoint),
}

fn compiled(input: ProjectInput) -> Option<(MemoryWorld, PagedDocument)> {
    let world = input.into_world(false);
    let document = typst::compile::<PagedDocument>(&world).output.ok()?;
    Some((world, document))
}

fn page_point(document: &PagedDocument, position: &PagedPosition) -> Option<PagePoint> {
    let size = document.pages().get(position.page.get() - 1)?.frame.size();
    if size.x <= Abs::zero() || size.y <= Abs::zero() {
        return None;
    }
    Some(PagePoint { page: position.page.get(), x: position.point.x / size.x, y: position.point.y / size.y })
}

pub type CompileFailure = Vec<(SourceDiagnostic, Option<std::ops::Range<usize>>)>;

const CACHE_TICK: std::time::Duration = std::time::Duration::from_secs(60);

pub(crate) fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name).ok().and_then(|v| v.trim().parse().ok()).unwrap_or(default)
}

pub(crate) fn resident_bytes() -> Option<usize> {
    let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
    let pages: usize = statm.split_whitespace().nth(1)?.parse().ok()?;
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    (page_size > 0).then(|| pages * page_size as usize)
}

pub(crate) fn cache_max_idle() -> std::time::Duration {
    CACHE_TICK * env_usize("TYPST_CACHE_MAX_IDLE_MINUTES", 60).max(1) as u32
}

const DEFAULT_CACHE_MEMORY_MB: usize = 2560;

fn container_memory_limit() -> Option<usize> {
    let limit = std::fs::read_to_string("/sys/fs/cgroup/memory.max").ok()?;
    limit.trim().parse().ok()
}

pub(crate) fn cache_memory_budget() -> usize {
    let default_budget = DEFAULT_CACHE_MEMORY_MB * 1_000_000;
    let default_budget = container_memory_limit().map_or(default_budget, |limit| default_budget.min(limit / 10 * 7));
    std::env::var("TYPST_CACHE_MAX_MEMORY_MB")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .map_or(default_budget, |megabytes| megabytes.saturating_mul(1_000_000))
}

pub fn spawn_cache_eviction() {
    let max_idle_ticks = env_usize("TYPST_CACHE_MAX_IDLE_MINUTES", 60).max(1);
    let max_memory = cache_memory_budget();
    tracing::info!(
        "Typst cache: entries dropped after {max_idle_ticks} min unused, or after 1 min above {} MB resident",
        max_memory / 1_000_000
    );
    std::thread::Builder::new()
        .name("typst-cache-eviction".into())
        .spawn(move || loop {
            std::thread::sleep(CACHE_TICK);
            let over_budget = resident_bytes().is_some_and(|rss| rss > max_memory);
            if over_budget {
                tracing::info!("Typst cache: over the memory budget, dropping entries unused for 1 min");
            }
            typst::comemo::evict(if over_budget { 1 } else { max_idle_ticks });
        })
        .expect("failed to start the Typst cache eviction thread");
}

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

pub enum JobError {
    Busy,
    TimedOut,
    Failed,
}

pub async fn run_blocking<T, F>(job: F) -> Result<T, JobError>
where
    F: FnOnce(&TypstCompiler) -> T + Send + 'static,
    T: Send + 'static,
{
    let permit = crate::compile::limits::acquire().await.map_err(|_| JobError::Busy)?;
    let started = std::time::Instant::now();
    let handle = tokio::task::spawn_blocking(move || {
        let output = job(&TypstCompiler);
        let elapsed = started.elapsed();
        if elapsed > crate::compile::limits::time_limit() {
            tracing::warn!("A Typst compile finished after {} s, past the time limit", elapsed.as_secs());
        }
        drop(permit);
        output
    });
    match tokio::time::timeout(crate::compile::limits::time_limit(), handle).await {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(error)) => {
            tracing::error!("Typst compilation task failed: {error}");
            Err(JobError::Failed)
        }
        Err(_) => {
            tracing::warn!("A Typst compile passed the {} s time limit and keeps its compile slot until it ends", crate::compile::limits::time_limit().as_secs());
            Err(JobError::TimedOut)
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
                let svgs = export_svg_pages(doc.pages(), &options);
                let thumbnail = svgs.first().cloned().unwrap_or_default();
                let notes = crate::compile::speaker_notes::page_notes(&world, &doc);
                Ok(Preview { svgs, thumbnail, stats, notes, outside_dependencies: world.outside_dependencies() })
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

    pub fn cursor_positions(&self, input: ProjectInput, path: &str, utf16_offset: usize) -> Vec<PagePoint> {
        let Some((world, document)) = compiled(input) else {
            return Vec::new();
        };
        let Some(source) = project_file_id(path).and_then(|id| world.source(id).ok()) else {
            return Vec::new();
        };
        let Some(cursor) = source.lines().utf16_to_byte(utf16_offset) else {
            return Vec::new();
        };
        typst_ide::jump_from_cursor(&document, &source, cursor)
            .iter()
            .filter_map(|position| page_point(&document, position))
            .collect()
    }

    pub fn click_target(&self, input: ProjectInput, page: usize, x: f64, y: f64) -> Option<ClickTarget> {
        let (world, document) = compiled(input)?;
        let size = document.pages().get(page.checked_sub(1)?)?.frame.size();
        let position = PagedPosition {
            page: std::num::NonZeroUsize::new(page)?,
            point: Point::new(size.x * x.clamp(0.0, 1.0), size.y * y.clamp(0.0, 1.0)),
        };
        match typst_ide::jump_from_click(&world, &document, &position)? {
            Jump::File(id, byte) => {
                let path = project_path(id)?;
                let offset = world.source(id).ok()?.lines().byte_to_utf16(byte)?;
                Some(ClickTarget::Source { path, offset })
            }
            Jump::Position(target) => page_point(&document, &target).map(ClickTarget::Page),
            Jump::Url(_) => None,
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
                pdf(&doc, &opts).output.map_err(|errors| {
                    use typst::WorldExt;
                    errors.into_iter().map(|d| {
                        let range = world.range(d.span);
                        (d, range)
                    }).collect()
                })
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

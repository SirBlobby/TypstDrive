use chrono::Datelike;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use typst::diag::{FileError, FileResult};
use typst::format::Format;
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::World;
use typst::{Library, LibraryExt};
use typst_kit::downloader::SystemDownloader;
use typst_kit::packages::SystemPackages;

use crate::compile::fonts::{shared_book, shared_fonts};
use crate::remote_hosts::resolve_remote_host;

fn shared_library(enable_html: bool) -> &'static typst::utils::LazyHash<Library> {
    static PLAIN: OnceLock<typst::utils::LazyHash<Library>> = OnceLock::new();
    static WITH_HTML: OnceLock<typst::utils::LazyHash<Library>> = OnceLock::new();
    let cell = if enable_html { &WITH_HTML } else { &PLAIN };
    cell.get_or_init(|| {
        let mut builder = Library::builder(export_formats());
        if enable_html {
            builder = builder.with_features([typst::Feature::Html].into_iter().collect());
        }
        typst::utils::LazyHash::new(builder.build())
    })
}

fn custom_book(custom_fonts: Vec<typst::text::FontInfo>) -> Option<typst::utils::LazyHash<FontBook>> {
    if custom_fonts.is_empty() {
        return None;
    }
    let mut book = shared_fonts().book();
    for info in custom_fonts {
        book.push(info);
    }
    Some(typst::utils::LazyHash::new(book))
}

pub struct MemoryWorld {
    library: &'static typst::utils::LazyHash<Library>,
    main: FileId,
    files: HashMap<String, Vec<u8>>,
    local_packages: HashMap<String, HashMap<String, Vec<u8>>>,
    custom_book: Option<typst::utils::LazyHash<FontBook>>,
    fonts: Vec<Font>,
    font_file_names: Vec<String>,
    packages: SystemPackages,
    accessed_paths: Mutex<HashSet<String>>,
    accessed_font_files: Mutex<HashSet<String>>,
    read_outside_input: AtomicBool,
    today_answers: Mutex<Vec<TodayAnswer>>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TodayAnswer {
    pub offset_seconds: Option<i32>,
    pub date: Option<(i32, u32, u32)>,
}

impl TodayAnswer {
    pub fn still_holds(&self) -> bool {
        today_date(self.offset_seconds) == self.date
    }
}

fn today_date(offset_seconds: Option<i32>) -> Option<(i32, u32, u32)> {
    let now = chrono::Local::now();
    let date = match offset_seconds {
        Some(seconds) => now.with_timezone(&chrono::FixedOffset::east_opt(seconds)?).date_naive(),
        None => now.date_naive(),
    };
    Some((date.year(), date.month(), date.day()))
}

#[derive(Default)]
pub struct UsedResources {
    pub paths: HashSet<String>,
    pub font_files: HashSet<String>,
}

pub const LOCAL_PACKAGE_NAMESPACE: &str = "typstdrive";

const REMOTE_FETCH_TIMEOUT_SECS: u64 = 15;
const REMOTE_MAX_BYTES: u64 = 50 * 1024 * 1024;
const REMOTE_MAX_REDIRECTS: u32 = 5;

fn normalize_path(path: &str) -> String {
    path.trim_start_matches('/').replace('\\', "/")
}

pub fn project_file_id(path: &str) -> Option<FileId> {
    VirtualPath::new(path).ok().map(|vpath| FileId::new(RootedPath::new(VirtualRoot::Project, vpath)))
}

pub fn project_path(id: FileId) -> Option<String> {
    matches!(id.root(), VirtualRoot::Project).then(|| normalize_path(id.vpath().get_without_slash()))
}

fn export_formats() -> [Format; 4] {
    [
        typst_html::FORMAT,
        typst_pdf::FORMAT,
        typst_svg::FORMAT,
        typst_render::FORMAT,
    ]
}

fn remote_url(path: &str) -> Option<String> {
    for scheme in ["https", "http"] {
        let prefix = format!("{scheme}:/");
        if let Some(rest) = path.strip_prefix(&prefix) {
            let host_and_path = rest.trim_start_matches('/');
            return Some(format!("{scheme}://{host_and_path}"));
        }
    }
    None
}

fn fetch_remote(url: &str) -> FileResult<Vec<u8>> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(REMOTE_FETCH_TIMEOUT_SECS))
        .redirects(REMOTE_MAX_REDIRECTS)
        .resolver(resolve_remote_host)
        .build();

    let response = agent
        .get(url)
        .call()
        .map_err(|e| FileError::Other(Some(format!("failed to fetch {url}: {e}").into())))?;

    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(REMOTE_MAX_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|e| FileError::Other(Some(format!("failed to read {url}: {e}").into())))?;

    Ok(bytes)
}

impl MemoryWorld {
    pub fn new_project(
        entrypoint: String,
        files: HashMap<String, Vec<u8>>,
        local_packages: HashMap<String, HashMap<String, Vec<u8>>>,
        enable_html: bool,
    ) -> Self {
        let main = project_file_id(&entrypoint).unwrap_or_else(|| project_file_id("main.typ").unwrap());
        let downloader = SystemDownloader::new("TypstDrive (typst-kit)");
        let packages = SystemPackages::new(downloader);

        let mut custom_fonts = Vec::new();
        let mut fonts = Vec::new();
        let mut font_file_names = Vec::new();

        // Add custom fonts from files, registered only by their embedded metadata
        // so that all variants (Bold, Italic, etc.) resolve correctly under one family name.
        let mut font_names: Vec<&String> = files
            .keys()
            .filter(|name| {
                let lower = name.to_lowercase();
                lower.ends_with(".ttf") || lower.ends_with(".otf")
            })
            .collect();
        font_names.sort();
        for name in font_names {
            for font in Font::iter(Bytes::new(files[name].clone())) {
                custom_fonts.push(font.info().clone());
                fonts.push(font);
                font_file_names.push(name.clone());
            }
        }

        Self {
            library: shared_library(enable_html),
            main,
            files,
            local_packages,
            custom_book: custom_book(custom_fonts),
            fonts,
            font_file_names,
            packages,
            accessed_paths: Mutex::new(HashSet::new()),
            accessed_font_files: Mutex::new(HashSet::new()),
            read_outside_input: AtomicBool::new(false),
            today_answers: Mutex::new(Vec::new()),
        }
    }

    pub fn outside_dependencies(&self) -> Option<Vec<TodayAnswer>> {
        if self.read_outside_input.load(Ordering::Relaxed) {
            return None;
        }
        Some(self.today_answers.lock().map(|answers| answers.clone()).unwrap_or_default())
    }

    pub fn used_resources(&self) -> UsedResources {
        UsedResources {
            paths: self.accessed_paths.lock().map(|paths| paths.clone()).unwrap_or_default(),
            font_files: self.accessed_font_files.lock().map(|fonts| fonts.clone()).unwrap_or_default(),
        }
    }

    fn load_bytes(&self, id: FileId) -> FileResult<Vec<u8>> {
        let path = normalize_path(id.vpath().get_without_slash());

        if let VirtualRoot::Package(package) = id.root() {
            if package.namespace.as_str() == LOCAL_PACKAGE_NAMESPACE {
                let key = format!("{}:{}", package.name, package.version);
                return self
                    .local_packages
                    .get(&key)
                    .and_then(|files| files.get(&path))
                    .cloned()
                    .ok_or_else(|| FileError::NotFound(path.clone().into()));
            }
            if package.namespace.as_str() != "preview" {
                self.read_outside_input.store(true, Ordering::Relaxed);
            }

            let root = self
                .packages
                .obtain(package)
                .map_err(|e| FileError::Other(Some(e.to_string().into())))?;
            return root.load(id.vpath()).map(|bytes| bytes.to_vec());
        }

        if let Some(url) = remote_url(&path) {
            self.read_outside_input.store(true, Ordering::Relaxed);
            return fetch_remote(&url);
        }

        let data = self.files.get(&path).cloned();
        if data.is_some() {
            if let Ok(mut accessed) = self.accessed_paths.lock() {
                accessed.insert(path.clone());
            }
        }
        data.ok_or_else(|| FileError::NotFound(path.into()))
    }
}

impl typst_ide::IdeWorld for MemoryWorld {
    fn upcast(&self) -> &dyn World {
        self
    }
}

impl World for MemoryWorld {
    fn library(&self) -> &typst::utils::LazyHash<Library> {
        self.library
    }

    fn book(&self) -> &typst::utils::LazyHash<FontBook> {
        self.custom_book.as_ref().unwrap_or_else(|| shared_book())
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        let data = self.load_bytes(id)?;
        let text = std::str::from_utf8(&data)
            .map_err(|_| FileError::InvalidUtf8)?
            .to_owned();
        Ok(Source::new(id, text))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let data = self.load_bytes(id)?;
        Ok(Bytes::new(data))
    }

    fn font(&self, index: usize) -> Option<Font> {
        let shared = shared_fonts();
        if index < shared.count() {
            return shared.font(index);
        }
        let custom_index = index - shared.count();
        if let (Some(name), Ok(mut accessed)) = (self.font_file_names.get(custom_index), self.accessed_font_files.lock()) {
            accessed.insert(name.clone());
        }
        self.fonts.get(custom_index).cloned()
    }

    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        let offset_seconds = offset.map(|offset| offset.hours() as i32 * 3600);
        let date = today_date(offset_seconds);
        if let Ok(mut answers) = self.today_answers.lock() {
            let answer = TodayAnswer { offset_seconds, date };
            if !answers.contains(&answer) {
                answers.push(answer);
            }
        }
        let (year, month, day) = date?;
        Datetime::from_ymd(year, month as u8, day as u8)
    }
}

pub struct OverlayWorld<'a> {
    base: &'a MemoryWorld,
    main: Source,
}

impl MemoryWorld {
    pub fn with_main(&self, path: &str, text: String) -> OverlayWorld<'_> {
        let vpath = VirtualPath::new(path).unwrap_or_else(|_| VirtualPath::new("main.typ").unwrap());
        let id = FileId::new(RootedPath::new(VirtualRoot::Project, vpath));
        OverlayWorld { base: self, main: Source::new(id, text) }
    }
}

impl World for OverlayWorld<'_> {
    fn library(&self) -> &typst::utils::LazyHash<Library> {
        self.base.library()
    }

    fn book(&self) -> &typst::utils::LazyHash<FontBook> {
        self.base.book()
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            return Ok(self.main.clone());
        }
        self.base.source(id)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.base.file(id)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.base.font(index)
    }

    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        self.base.today(offset)
    }
}

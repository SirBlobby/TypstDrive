use chrono::Datelike;
use std::collections::HashMap;

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::World;
use typst::{Library, LibraryExt};
use typst_kit::downloader::SystemDownloader;
use typst_kit::packages::SystemPackages;

pub struct MemoryWorld {
    library: typst::utils::LazyHash<Library>,
    main: FileId,
    files: HashMap<String, Vec<u8>>,
    local_packages: HashMap<String, HashMap<String, Vec<u8>>>,
    book: typst::utils::LazyHash<FontBook>,
    fonts: Vec<Font>,
    packages: SystemPackages,
}

const LOCAL_NAMESPACE: &str = "typstdrive";

fn normalize_path(path: &str) -> String {
    path.trim_start_matches('/').replace('\\', "/")
}

impl MemoryWorld {
    pub fn new_project(
        entrypoint: String,
        files: HashMap<String, Vec<u8>>,
        local_packages: HashMap<String, HashMap<String, Vec<u8>>>,
    ) -> Self {
        let main = FileId::new(RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new(&entrypoint).unwrap_or_else(|_| VirtualPath::new("main.typ").unwrap()),
        ));
        let downloader = SystemDownloader::new("TypstDrive (typst-kit)");
        let packages = SystemPackages::new(downloader);

        let mut book = FontBook::new();
        let mut fonts = Vec::new();

        // Add embedded fonts
        for data in typst_assets::fonts() {
            let buffer = Bytes::new(data);
            for font in Font::iter(buffer) {
                book.push(font.info().clone());
                fonts.push(font);
            }
        }

        // Add custom fonts from files, registered only by their embedded metadata
        // so that all variants (Bold, Italic, etc.) resolve correctly under one family name.
        for (name, data) in &files {
            if name.to_lowercase().ends_with(".ttf") || name.to_lowercase().ends_with(".otf") {
                for font in Font::iter(Bytes::new(data.clone())) {
                    book.push(font.info().clone());
                    fonts.push(font);
                }
            }
        }

        Self {
            library: typst::utils::LazyHash::new(Library::builder().build()),
            main,
            files,
            local_packages,
            book: typst::utils::LazyHash::new(book),
            fonts,
            packages,
        }
    }

    fn load_bytes(&self, id: FileId) -> FileResult<Vec<u8>> {
        let path = normalize_path(id.vpath().get_without_slash());

        if let VirtualRoot::Package(package) = id.root() {
            if package.namespace.as_str() == LOCAL_NAMESPACE {
                let key = format!("{}:{}", package.name, package.version);
                return self
                    .local_packages
                    .get(&key)
                    .and_then(|files| files.get(&path))
                    .cloned()
                    .ok_or_else(|| FileError::NotFound(path.clone().into()));
            }

            let root = self
                .packages
                .obtain(package)
                .map_err(|e| FileError::Other(Some(e.to_string().into())))?;
            return root.load(id.vpath()).map(|bytes| bytes.to_vec());
        }

        self.files
            .get(&path)
            .cloned()
            .ok_or_else(|| FileError::NotFound(path.into()))
    }
}

impl World for MemoryWorld {
    fn library(&self) -> &typst::utils::LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &typst::utils::LazyHash<FontBook> {
        &self.book
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
        self.fonts.get(index).cloned()
    }

    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        let now = chrono::Local::now();
        let date = if let Some(offset) = offset {
            let offset_secs = offset.hours() as i32 * 3600;
            let offset_chrono = chrono::FixedOffset::east_opt(offset_secs)?;
            now.with_timezone(&offset_chrono).date_naive()
        } else {
            now.date_naive()
        };

        Datetime::from_ymd(date.year(), date.month() as u8, date.day() as u8)
    }
}

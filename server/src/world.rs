use chrono::Datelike;
use std::collections::HashMap;

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst_kit::download::{Downloader, ProgressSink};
use typst_kit::package::PackageStorage;
use typst::text::{Font, FontBook};
use typst::World;
use typst::{Library, LibraryExt};

pub struct MemoryWorld {
    library: typst::utils::LazyHash<Library>,
    main: FileId,
    source: Source,
    files: HashMap<String, Vec<u8>>,
    book: typst::utils::LazyHash<FontBook>,
    fonts: Vec<Font>,
    packages: PackageStorage,
}

impl MemoryWorld {
    pub fn new(text: String, files: HashMap<String, Vec<u8>>) -> Self {
        let main = FileId::new(None, VirtualPath::new("main.typ"));
        let source = Source::new(main, text);
        let downloader = Downloader::new("TypstDrive (typst-kit)");
        let packages = PackageStorage::new(None, None, downloader);

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

        // Add custom fonts from files
        for (name, data) in &files {
            if name.ends_with(".ttf") || name.ends_with(".otf") {
                for font in Font::iter(Bytes::new(data.clone())) {
                    let info = font.info().clone();
                    book.push(info.clone());
                    fonts.push(font.clone());

                    let mut custom_info = info;
                    if let Some(stem) = std::path::Path::new(name).file_stem() {
                        if let Some(stem_str) = stem.to_str() {
                            custom_info.family = stem_str.to_string();
                            book.push(custom_info);
                            fonts.push(font);
                        }
                    }
                }
            }
        }

        Self {
            library: typst::utils::LazyHash::new(Library::builder().build()),
            main,
            source,
            files,
            book: typst::utils::LazyHash::new(book),
            fonts,
            packages,
        }
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
        if id == self.main {
            Ok(self.source.clone())
        } else if let Some(package) = id.package() {
            let dir = self
                .packages
                .prepare_package(package, &mut ProgressSink)
                .map_err(|e| FileError::Other(Some(e.to_string().into())))?;
            let path = id.vpath().resolve(&dir).ok_or_else(|| FileError::NotFound(id.vpath().as_rootless_path().into()))?;
            let data = std::fs::read(&path).map_err(|_| FileError::NotFound(id.vpath().as_rootless_path().into()))?;
            let text = String::from_utf8(data).map_err(|_| FileError::InvalidUtf8)?;
            Ok(Source::new(id, text))
        } else {
            Err(FileError::NotFound(
                id.vpath().as_rootless_path().into(),
            ))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main {
            Ok(Bytes::from_string(self.source.text().to_string()))
        } else if let Some(package) = id.package() {
            let dir = self
                .packages
                .prepare_package(package, &mut ProgressSink)
                .map_err(|e| FileError::Other(Some(e.to_string().into())))?;
            let path = id.vpath().resolve(&dir).ok_or_else(|| FileError::NotFound(id.vpath().as_rootless_path().into()))?;
            let data = std::fs::read(&path).map_err(|_| FileError::NotFound(id.vpath().as_rootless_path().into()))?;
            Ok(Bytes::new(data))
        } else if let Some(data) = self.files.get(&id.vpath().as_rootless_path().to_string_lossy().to_string().replace("\\", "/")) {
            Ok(Bytes::new(data.clone()))
        } else {
            Err(FileError::NotFound(
                id.vpath().as_rootless_path().into(),
            ))
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let now = chrono::Local::now();
        let date = if let Some(offset) = offset {
            let offset = chrono::FixedOffset::east_opt(offset as i32)?;
            now.with_timezone(&offset).date_naive()
        } else {
            now.date_naive()
        };

        Datetime::from_ymd(date.year(), date.month() as u8, date.day() as u8)
    }
}

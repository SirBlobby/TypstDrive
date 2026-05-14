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
    source: Source,
    files: HashMap<String, Vec<u8>>,
    book: typst::utils::LazyHash<FontBook>,
    fonts: Vec<Font>,
    packages: SystemPackages,
}

impl MemoryWorld {
    pub fn new(text: String, files: HashMap<String, Vec<u8>>) -> Self {
        let main = FileId::new(RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new("main.typ").unwrap(),
        ));
        let source = Source::new(main, text);
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
        } else if let VirtualRoot::Package(package) = id.root() {
            let root = self
                .packages
                .obtain(package)
                .map_err(|e| FileError::Other(Some(e.to_string().into())))?;
            let data = root.load(id.vpath())?;
            let text = std::str::from_utf8(&data)
                .map_err(|_| FileError::InvalidUtf8)?
                .to_owned();
            Ok(Source::new(id, text))
        } else {
            Err(FileError::NotFound(id.vpath().get_without_slash().into()))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main {
            Ok(Bytes::from_string(self.source.text().to_string()))
        } else if let VirtualRoot::Package(package) = id.root() {
            let root = self
                .packages
                .obtain(package)
                .map_err(|e| FileError::Other(Some(e.to_string().into())))?;
            root.load(id.vpath())
        } else if let Some(data) = self.files.get(
            &id.vpath()
                .get_without_slash()
                .to_string()
                .replace("\\", "/"),
        ) {
            Ok(Bytes::new(data.clone()))
        } else {
            Err(FileError::NotFound(id.vpath().get_without_slash().into()))
        }
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

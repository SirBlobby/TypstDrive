use chrono::Datelike;
use std::collections::HashMap;

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::World;
use typst::{Library, LibraryExt};
use typst_kit::downloader::SystemDownloader;
use typst_kit::fonts::FontStore;
use typst_kit::packages::SystemPackages;

pub struct MemoryWorld {
    library: typst::utils::LazyHash<Library>,
    main: FileId,
    source: Source,
    files: HashMap<String, Vec<u8>>,
    fonts: std::sync::LazyLock<FontStore, Box<dyn Fn() -> FontStore + Send + Sync>>,
    packages: SystemPackages,
}

impl MemoryWorld {
    pub fn new(text: String, files: HashMap<String, Vec<u8>>) -> Self {
        let main = FileId::new(RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new("main.typ").unwrap(),
        ));
        let source = Source::new(main, text);
        let files_clone = files.clone();
        let downloader = SystemDownloader::new("TypstDrive (typst-kit)");
        let packages = SystemPackages::new(downloader);

        Self {
            library: typst::utils::LazyHash::new(Library::builder().build()),
            main,
            source,
            fonts: std::sync::LazyLock::new(Box::new(move || {
                let mut store = FontStore::new();
                store.extend(typst_kit::fonts::embedded());

                for (name, data) in &files {
                    if name.ends_with(".ttf") || name.ends_with(".otf") {
                        for font in Font::iter(Bytes::new(data.clone())) {
                            let info = font.info().clone();
                            store.push((font.clone(), info.clone()));

                            let mut custom_info = info;
                            if let Some(stem) = std::path::Path::new(name).file_stem() {
                                if let Some(stem_str) = stem.to_str() {
                                    custom_info.family = stem_str.to_string();
                                    store.push((font, custom_info));
                                }
                            }
                        }
                    }
                }

                store
            })),
            files: files_clone,
            packages,
        }
    }
}

impl World for MemoryWorld {
    fn library(&self) -> &typst::utils::LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &typst::utils::LazyHash<FontBook> {
        self.fonts.book()
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main {
            Ok(self.source.clone())
        } else if let typst::syntax::VirtualRoot::Package(package) = id.root() {
            let root = self
                .packages
                .obtain(package)
                .map_err(|e| FileError::Other(Some(e.to_string().into())))?;
            let data = root.load(id.vpath())?;
            let text = String::from_utf8(data.to_vec()).map_err(|_| FileError::InvalidUtf8)?;
            Ok(Source::new(id, text))
        } else {
            Err(FileError::NotFound(
                std::path::Path::new(id.vpath().get_without_slash()).into(),
            ))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main {
            Ok(Bytes::from_string(self.source.text().to_string()))
        } else if let typst::syntax::VirtualRoot::Package(package) = id.root() {
            let root = self
                .packages
                .obtain(package)
                .map_err(|e| FileError::Other(Some(e.to_string().into())))?;
            root.load(id.vpath())
        } else if let Some(data) = self.files.get(id.vpath().get_without_slash()) {
            Ok(Bytes::new(data.clone()))
        } else {
            Err(FileError::NotFound(
                std::path::Path::new(id.vpath().get_without_slash()).into(),
            ))
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }

    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        let now = chrono::Local::now();
        let date = if let Some(offset) = offset {
            let offset = chrono::FixedOffset::east_opt(offset.seconds() as i32)?;
            now.with_timezone(&offset).date_naive()
        } else {
            now.date_naive()
        };

        Datetime::from_ymd(date.year(), date.month() as u8, date.day() as u8)
    }
}

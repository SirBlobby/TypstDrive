use chrono::Datelike;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::Mutex;

use typst::diag::{FileError, FileResult};
use typst::format::Format;
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::World;
use typst::{Library, LibraryExt};
use typst_kit::downloader::SystemDownloader;
use typst_kit::packages::SystemPackages;

use crate::compile::fonts::shared_fonts;

pub struct MemoryWorld {
    library: typst::utils::LazyHash<Library>,
    main: FileId,
    files: HashMap<String, Vec<u8>>,
    local_packages: HashMap<String, HashMap<String, Vec<u8>>>,
    book: typst::utils::LazyHash<FontBook>,
    fonts: Vec<Font>,
    font_file_names: Vec<String>,
    packages: SystemPackages,
    accessed_paths: Mutex<HashSet<String>>,
    accessed_font_files: Mutex<HashSet<String>>,
}

#[derive(Default)]
pub struct UsedResources {
    pub paths: HashSet<String>,
    pub font_files: HashSet<String>,
}

const LOCAL_NAMESPACE: &str = "typstdrive";

const REMOTE_FETCH_TIMEOUT_SECS: u64 = 15;
const REMOTE_MAX_BYTES: u64 = 50 * 1024 * 1024;
const REMOTE_MAX_REDIRECTS: u32 = 5;

fn normalize_path(path: &str) -> String {
    path.trim_start_matches('/').replace('\\', "/")
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

fn allows_private_addresses() -> bool {
    std::env::var("REMOTE_URLS_ALLOW_PRIVATE").map(|value| value.eq_ignore_ascii_case("true") || value == "1").unwrap_or(false)
}

fn is_public_address(address: &IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => {
            let [first, second, ..] = address.octets();
            let shared_address_space = first == 100 && (second & 0xc0) == 64;
            !(address.is_private()
                || address.is_loopback()
                || address.is_link_local()
                || address.is_broadcast()
                || address.is_documentation()
                || address.is_unspecified()
                || address.is_multicast()
                || first == 0
                || first >= 240
                || shared_address_space)
        }
        IpAddr::V6(address) => {
            if let Some(mapped) = address.to_ipv4_mapped() {
                return is_public_address(&IpAddr::V4(mapped));
            }
            let first_segment = address.segments()[0];
            let unique_local = (first_segment & 0xfe00) == 0xfc00;
            let link_local = (first_segment & 0xffc0) == 0xfe80;
            !(address.is_loopback() || address.is_unspecified() || address.is_multicast() || unique_local || link_local)
        }
    }
}

fn resolve_remote_host(host_and_port: &str) -> std::io::Result<Vec<SocketAddr>> {
    let addresses: Vec<SocketAddr> = host_and_port.to_socket_addrs()?.collect();
    if allows_private_addresses() {
        return Ok(addresses);
    }
    let public: Vec<SocketAddr> = addresses.into_iter().filter(|address| is_public_address(&address.ip())).collect();
    if public.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "remote files must be on a public address",
        ));
    }
    Ok(public)
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
        let main = FileId::new(RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new(&entrypoint).unwrap_or_else(|_| VirtualPath::new("main.typ").unwrap()),
        ));
        let downloader = SystemDownloader::new("TypstDrive (typst-kit)");
        let packages = SystemPackages::new(downloader);

        let mut book = shared_fonts().book();
        let mut fonts = Vec::new();
        let mut font_file_names = Vec::new();

        // Add custom fonts from files, registered only by their embedded metadata
        // so that all variants (Bold, Italic, etc.) resolve correctly under one family name.
        for (name, data) in &files {
            if name.to_lowercase().ends_with(".ttf") || name.to_lowercase().ends_with(".otf") {
                for font in Font::iter(Bytes::new(data.clone())) {
                    book.push(font.info().clone());
                    fonts.push(font);
                    font_file_names.push(name.clone());
                }
            }
        }

        let mut library_builder = Library::builder(export_formats());
        if enable_html {
            library_builder =
                library_builder.with_features([typst::Feature::Html].into_iter().collect());
        }
        let library = library_builder.build();

        Self {
            library: typst::utils::LazyHash::new(library),
            main,
            files,
            local_packages,
            book: typst::utils::LazyHash::new(book),
            fonts,
            font_file_names,
            packages,
            accessed_paths: Mutex::new(HashSet::new()),
            accessed_font_files: Mutex::new(HashSet::new()),
        }
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

        if let Some(url) = remote_url(&path) {
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

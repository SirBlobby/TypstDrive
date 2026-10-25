use std::path::{Component, Path};

use axum::http::StatusCode;
use typst::syntax::package::PackageSpec;
use typst_kit::downloader::SystemDownloader;
use typst_kit::packages::SystemPackages;

use super::index::Template;
use super::{internal_error, ApiError};

const MAX_FILES: usize = 200;
const MAX_TOTAL_BYTES: usize = 25 * 1024 * 1024;

pub struct TemplateFile {
    pub path: String,
    pub data: Vec<u8>,
}

fn is_safe_relative(path: &str) -> bool {
    Path::new(path)
        .components()
        .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

fn too_large() -> ApiError {
    (StatusCode::BAD_REQUEST, format!("This template has more than {MAX_FILES} files or 25 MB of files"))
}

fn collect(directory: &Path, prefix: &str, files: &mut Vec<TemplateFile>, total_bytes: &mut usize) -> Result<(), ApiError> {
    let mut entries: Vec<_> = std::fs::read_dir(directory).map_err(internal_error)?.filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let file_type = entry.file_type().map_err(internal_error)?;
        let name = entry.file_name().to_string_lossy().to_string();
        let path = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };

        if file_type.is_dir() {
            collect(&entry.path(), &path, files, total_bytes)?;
        } else if file_type.is_file() {
            let data = std::fs::read(entry.path()).map_err(internal_error)?;
            *total_bytes += data.len();
            if files.len() >= MAX_FILES || *total_bytes > MAX_TOTAL_BYTES {
                return Err(too_large());
            }
            files.push(TemplateFile { path, data });
        }
    }
    Ok(())
}

pub async fn template_files(template: &Template) -> Result<Vec<TemplateFile>, ApiError> {
    if !is_safe_relative(&template.template_path) || !is_safe_relative(&template.entrypoint) {
        return Err((StatusCode::BAD_REQUEST, "This template has an invalid layout".to_string()));
    }
    let spec_text = format!("@preview/{}:{}", template.name, template.version);
    let template_path = template.template_path.clone();

    tokio::task::spawn_blocking(move || {
        let spec: PackageSpec = spec_text
            .parse()
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid template name".to_string()))?;
        let packages = SystemPackages::new(SystemDownloader::new("TypstDrive (typst-kit)"));
        let root = packages
            .obtain(&spec)
            .map_err(|error| (StatusCode::BAD_GATEWAY, format!("Could not download the template: {error}")))?;

        let mut files = Vec::new();
        let mut total_bytes = 0;
        collect(&root.path().join(&template_path), "", &mut files, &mut total_bytes)?;
        if files.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "This template has no files".to_string()));
        }
        Ok(files)
    })
    .await
    .map_err(internal_error)?
}

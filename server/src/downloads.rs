use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Write};

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::SignedCookieJar;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::models::Project;
use crate::projects::{assemble_project, decode_text_blob};
use crate::AppState;

type ApiError = (StatusCode, String);

enum ArchiveEntry {
    Directory(String),
    File(String, Vec<u8>),
}

#[derive(Default)]
struct NameAllocator {
    taken: HashSet<String>,
}

impl NameAllocator {
    fn claim(&mut self, name: &str) -> String {
        if self.taken.insert(name.to_lowercase()) {
            return name.to_string();
        }
        let (stem, extension) = match name.rfind('.') {
            Some(dot) if dot > 0 => (&name[..dot], &name[dot..]),
            _ => (name, ""),
        };
        let mut copy_number = 2;
        loop {
            let candidate = format!("{stem} ({copy_number}){extension}");
            if self.taken.insert(candidate.to_lowercase()) {
                return candidate;
            }
            copy_number += 1;
        }
    }
}

fn internal_error(error: impl std::fmt::Display) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn safe_segment(name: &str, fallback: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|character| match character {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            character if character.is_control() => '_',
            character => character,
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.').trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

fn safe_relative_path(path: &str) -> String {
    path.split(['/', '\\'])
        .filter(|segment| !segment.is_empty() && *segment != "." && *segment != "..")
        .map(|segment| safe_segment(segment, "file"))
        .collect::<Vec<_>>()
        .join("/")
}

async fn project_entries(state: &AppState, project: &Project, prefix: &str) -> Result<Vec<ArchiveEntry>, ApiError> {
    let rows = sqlx::query_as::<_, (String, String, Option<Vec<u8>>)>(
        "SELECT path, kind, content FROM project_files WHERE project_id = $1 ORDER BY path ASC"
    )
    .bind(&project.id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let mut entries = vec![ArchiveEntry::Directory(prefix.to_string())];
    let mut project_paths = HashSet::new();
    for (path, kind, content) in rows {
        let relative_path = safe_relative_path(&path);
        if relative_path.is_empty() {
            continue;
        }
        let bytes = content.unwrap_or_default();
        let data = if kind == "binary" { bytes } else { decode_text_blob(&bytes).into_bytes() };
        project_paths.insert(path);
        entries.push(ArchiveEntry::File(format!("{prefix}{relative_path}"), data));
    }

    let input = assemble_project(state, project, HashMap::new()).await;
    let used = {
        let compiler = state.compiler.lock().await;
        compiler.used_resources(input)
    };

    let account_files = sqlx::query_as::<_, (String, Vec<u8>)>("SELECT name, data FROM files WHERE owner_id = $1")
        .bind(&project.owner_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;

    let mut included = HashSet::new();
    for (name, data) in account_files {
        let is_used = used.paths.contains(&name) || used.font_files.contains(&name);
        if is_used && !project_paths.contains(&name) && included.insert(name.clone()) {
            entries.push(ArchiveEntry::File(format!("{prefix}{}", safe_relative_path(&name)), data));
        }
    }

    Ok(entries)
}

async fn folder_entries(state: &AppState, root_id: &str, root_name: &str, owner_id: &str) -> Result<Vec<ArchiveEntry>, ApiError> {
    let mut entries = Vec::new();
    let mut pending = vec![(root_id.to_string(), format!("{}/", safe_segment(root_name, "folder")))];

    while let Some((folder_id, prefix)) = pending.pop() {
        entries.push(ArchiveEntry::Directory(prefix.clone()));
        let mut names = NameAllocator::default();

        let subfolders = sqlx::query_as::<_, (String, String)>(
            "SELECT id, name FROM folders WHERE parent_id = $1 AND owner_id = $2 ORDER BY name ASC"
        )
        .bind(&folder_id)
        .bind(owner_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
        for (subfolder_id, name) in subfolders {
            let directory = names.claim(&safe_segment(&name, "folder"));
            pending.push((subfolder_id, format!("{prefix}{directory}/")));
        }

        let documents = sqlx::query_as::<_, (String, Option<Vec<u8>>)>(
            "SELECT title, content FROM documents WHERE folder_id = $1 AND owner_id = $2 ORDER BY title ASC"
        )
        .bind(&folder_id)
        .bind(owner_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
        for (title, content) in documents {
            let file_name = names.claim(&format!("{}.typ", safe_segment(&title, "document")));
            let text = decode_text_blob(content.as_deref().unwrap_or_default());
            entries.push(ArchiveEntry::File(format!("{prefix}{file_name}"), text.into_bytes()));
        }

        let files = sqlx::query_as::<_, (String, Vec<u8>)>(
            "SELECT name, data FROM files WHERE folder_id = $1 AND owner_id = $2 ORDER BY name ASC"
        )
        .bind(&folder_id)
        .bind(owner_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
        for (name, data) in files {
            let file_name = names.claim(&safe_segment(&name, "file"));
            entries.push(ArchiveEntry::File(format!("{prefix}{file_name}"), data));
        }

        let projects = sqlx::query_as::<_, Project>(
            "SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at FROM projects WHERE folder_id = $1 AND owner_id = $2 ORDER BY name ASC"
        )
        .bind(&folder_id)
        .bind(owner_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
        for project in projects {
            let directory = names.claim(&safe_segment(&project.name, "project"));
            entries.extend(project_entries(state, &project, &format!("{prefix}{directory}/")).await?);
        }
    }

    Ok(entries)
}

fn build_zip(entries: Vec<ArchiveEntry>) -> zip::result::ZipResult<Vec<u8>> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for entry in entries {
        match entry {
            ArchiveEntry::Directory(path) => writer.add_directory(path, options)?,
            ArchiveEntry::File(path, data) => {
                writer.start_file(path, options)?;
                writer.write_all(&data)?;
            }
        }
    }
    Ok(writer.finish()?.into_inner())
}

fn content_disposition(name: &str) -> String {
    let ascii_name: String = name
        .chars()
        .map(|character| if character.is_ascii_alphanumeric() || " -_.()".contains(character) { character } else { '_' })
        .collect();
    let encoded_name: String = name
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || b"-_.".contains(&byte) {
                (byte as char).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect();
    format!("attachment; filename=\"{ascii_name}.zip\"; filename*=UTF-8''{encoded_name}.zip")
}

async fn zip_response(entries: Vec<ArchiveEntry>, name: &str) -> Result<Response, ApiError> {
    let archive = tokio::task::spawn_blocking(move || build_zip(entries))
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/zip".to_string()),
            (header::CONTENT_DISPOSITION, content_disposition(name)),
        ],
        archive,
    )
        .into_response())
}

pub async fn download_folder(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Response, ApiError> {
    let user_id = jar
        .get("session_user_id")
        .map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let (folder_name,) = sqlx::query_as::<_, (String,)>("SELECT name FROM folders WHERE id = $1 AND owner_id = $2")
        .bind(&id)
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Folder not found".to_string()))?;

    let entries = folder_entries(&state, &id, &folder_name, &user_id).await?;
    zip_response(entries, &safe_segment(&folder_name, "folder")).await
}

pub async fn download_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Response, ApiError> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string());
    let (project, _) = crate::access::project_access(&state, &id, &user_id)
        .await
        .ok_or_else(crate::access::forbidden)?;

    let name = safe_segment(&project.name, "project");
    let entries = project_entries(&state, &project, &format!("{name}/")).await?;
    zip_response(entries, &name).await
}

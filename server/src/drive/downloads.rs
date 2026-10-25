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
use crate::drive::projects::{assemble_project, decode_text_blob, ProjectReader};
use crate::AppState;
use crate::db::sql;

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

async fn project_entries(state: &AppState, project: &Project, prefix: &str, reader: ProjectReader<'_>) -> Result<Vec<ArchiveEntry>, ApiError> {
    let rows = sqlx::query_as::<_, (String, String, Option<Vec<u8>>)>(
        sql("SELECT path, kind, content FROM project_files WHERE project_id = $1 ORDER BY path ASC")
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

    if !matches!(reader, ProjectReader::Owner) {
        return Ok(entries);
    }

    let input = assemble_project(state, project, HashMap::new(), reader).await;
    let used = crate::compile::compiler::run_blocking(move |compiler| compiler.used_resources(input))
        .await
        .unwrap_or_default();

    let account_files = sqlx::query_as::<_, (String, String)>(sql("SELECT id, name FROM files WHERE owner_id = $1 AND deleted_at IS NULL"))
        .bind(&project.owner_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;

    let mut included = HashSet::new();
    let used_files: Vec<(String, String)> = account_files
        .into_iter()
        .filter(|(_, name)| used.paths.contains(name) || used.font_files.contains(name))
        .filter(|(_, name)| !project_paths.contains(name) && included.insert(name.clone()))
        .collect();
    let used_ids: Vec<String> = used_files.iter().map(|(id, _)| id.clone()).collect();
    let mut loaded = crate::drive::file_data::load_many(state, &used_ids).await;
    for (id, name) in used_files {
        if let Some(data) = loaded.remove(&id) {
            entries.push(ArchiveEntry::File(format!("{prefix}{}", safe_relative_path(&name)), data));
        }
    }

    Ok(entries)
}

async fn folder_entries(state: &AppState, root_id: &str, root_name: &str, owner_id: &str, reader: ProjectReader<'_>) -> Result<Vec<ArchiveEntry>, ApiError> {
    let mut entries = Vec::new();
    let mut pending = vec![(root_id.to_string(), format!("{}/", safe_segment(root_name, "folder")))];

    while let Some((folder_id, prefix)) = pending.pop() {
        entries.push(ArchiveEntry::Directory(prefix.clone()));
        let mut names = NameAllocator::default();

        let subfolders = sqlx::query_as::<_, (String, String)>(
            sql("SELECT id, name FROM folders WHERE parent_id = $1 AND owner_id = $2 AND deleted_at IS NULL ORDER BY name ASC")
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

        let documents = sqlx::query_as::<_, (String, Option<Vec<u8>>, Option<String>)>(
            sql("SELECT title, content, language FROM documents WHERE folder_id = $1 AND owner_id = $2 AND deleted_at IS NULL ORDER BY title ASC")
        )
        .bind(&folder_id)
        .bind(owner_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
        for (title, content, language) in documents {
            let extension = crate::compile::latex::file_extension(language.as_deref());
            let file_name = names.claim(&format!("{}.{extension}", safe_segment(&title, "document")));
            let text = decode_text_blob(content.as_deref().unwrap_or_default());
            entries.push(ArchiveEntry::File(format!("{prefix}{file_name}"), text.into_bytes()));
        }

        let files = sqlx::query_as::<_, (String, Vec<u8>, Option<String>)>(
            sql("SELECT name, data, storage_key FROM files WHERE folder_id = $1 AND owner_id = $2 AND deleted_at IS NULL ORDER BY name ASC")
        )
        .bind(&folder_id)
        .bind(owner_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
        for (name, data) in state.file_storage.load_named(files).await? {
            let file_name = names.claim(&safe_segment(&name, "file"));
            entries.push(ArchiveEntry::File(format!("{prefix}{file_name}"), data));
        }

        let projects = sqlx::query_as::<_, Project>(
            sql("SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at FROM projects WHERE folder_id = $1 AND owner_id = $2 AND deleted_at IS NULL ORDER BY name ASC")
        )
        .bind(&folder_id)
        .bind(owner_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
        for project in projects {
            let directory = names.claim(&safe_segment(&project.name, "project"));
            entries.extend(project_entries(state, &project, &format!("{prefix}{directory}/"), reader).await?);
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

    crate::sharing::access::require_folder_access(&state, &id, &user_id).await?;
    let (folder_name, owner_id) = sqlx::query_as::<_, (String, String)>(sql("SELECT name, owner_id FROM folders WHERE id = $1 AND deleted_at IS NULL"))
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Folder not found".to_string()))?;

    let reader = if owner_id == user_id { ProjectReader::Owner } else { ProjectReader::Collaborator(Some(user_id.as_str())) };
    let entries = folder_entries(&state, &id, &folder_name, &owner_id, reader).await?;
    zip_response(entries, &safe_segment(&folder_name, "folder")).await
}

pub async fn download_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Response, ApiError> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string());
    let (project, role) = crate::sharing::access::project_access(&state, &id, &user_id)
        .await
        .ok_or_else(crate::sharing::access::forbidden)?;

    let name = safe_segment(&project.name, "project");
    let reader = ProjectReader::for_role(role, &user_id);
    let entries = project_entries(&state, &project, &format!("{name}/"), reader).await?;
    zip_response(entries, &name).await
}

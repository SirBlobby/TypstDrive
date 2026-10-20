use axum::{
    extract::{Path, Query, State, Multipart},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use yrs::{Doc, GetString, ReadTxn, StateVector, Text, Transact};
use yrs::updates::decoder::Decode;
use yrs::Update;

use crate::{
    sharing::access::{inherited_role, require_folder_access, require_owned_folder, strongest_role, AccessRole},
    compile::compiler::ProjectInput,
    drive::project_paths::clean_project_path,
    desktop::devices::{notify_devices, DeviceEvent},
    models::{
        CreateProjectFileRequest, CreateProjectRequest, Project, ProjectFile, UpdateProjectFileRequest,
        UpdateProjectRequest,
    },
    AppState,
};
use crate::db::sql;

const TEXT_NAME: &str = "typst";

pub fn encode_text_blob(text: &str) -> Vec<u8> {
    let doc = Doc::new();
    let handle = doc.get_or_insert_text(TEXT_NAME);
    handle.insert(&mut doc.transact_mut(), 0, text);
    let bytes = doc.transact().encode_state_as_update_v1(&StateVector::default());
    bytes
}

pub fn decode_text_blob(blob: &[u8]) -> String {
    let doc = Doc::new();
    if let Ok(update) = Update::decode_v1(blob) {
        doc.transact_mut().apply_update(update);
    }
    let handle = doc.get_or_insert_text(TEXT_NAME);
    let text = handle.get_string(&doc.transact());
    text
}

fn is_text_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    [".typ", ".toml", ".bib", ".csl", ".yml", ".yaml", ".json", ".md", ".txt", ".csv"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}

fn default_manifest(name: &str) -> String {
    format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nentrypoint = \"main.typ\"\nauthors = [\"Anonymous\"]\nlicense = \"MIT\"\ndescription = \"\"\n"
    )
}

fn slugify(name: &str) -> String {
    let slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let trimmed = slug.trim_matches('-').replace("--", "-");
    if trimmed.is_empty() {
        "my-project".to_string()
    } else {
        trimmed
    }
}

fn can_edit_role(role: &str) -> bool {
    matches!(role, "owner" | "editor")
}

fn read_only() -> (StatusCode, String) {
    (StatusCode::FORBIDDEN, "Read-only access".to_string())
}

pub async fn project_role(
    state: &AppState,
    project_id: &str,
    user_id_opt: &Option<String>,
) -> Option<(Project, String)> {
    let project = sqlx::query_as::<_, Project>(
        sql("SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, link_password_hash, link_expires_at, created_at, updated_at FROM projects WHERE id = $1")
    )
    .bind(project_id)
    .fetch_optional(&state.db)
    .await
    .ok()??;

    let mut role = None;
    if let Some(uid) = user_id_opt {
        if &project.owner_id == uid {
            return Some((project, "owner".to_string()));
        }
        role = sqlx::query_as::<_, (String,)>(
            sql("SELECT role FROM project_collaborators WHERE project_id = $1 AND user_id = $2"),
        )
        .bind(project_id)
        .bind(uid)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .and_then(|(role,)| AccessRole::from_share_role(&role));
        role = strongest_role(role, inherited_role(state, &project.folder_id, &project.owner_id, user_id_opt).await);
        role = strongest_role(role, crate::teams::content_role(state, &project.owner_id, uid).await);
    }

    let public_role = crate::sharing::links::public_access(
        &project.id,
        project.public_role.as_deref(),
        project.link_password_hash.as_deref(),
        project.link_expires_at.as_deref(),
    );
    strongest_role(role, public_role).map(|role| (project, role.as_str().to_string()))
}

pub async fn assemble_project(
    state: &AppState,
    project: &Project,
    overrides: HashMap<String, String>,
) -> ProjectInput {
    let rows = sqlx::query_as::<_, (String, String, Option<Vec<u8>>)>(
        sql("SELECT path, kind, content FROM project_files WHERE project_id = $1"),
    )
    .bind(&project.id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut files: HashMap<String, Vec<u8>> = HashMap::new();
    let mut sources: Vec<String> = Vec::new();
    for (path, kind, content) in rows {
        if let Some(live) = overrides.get(&path) {
            sources.push(live.clone());
            files.insert(path, live.clone().into_bytes());
        } else if kind == "binary" {
            files.insert(path, content.unwrap_or_default());
        } else {
            let text = decode_text_blob(&content.unwrap_or_default());
            sources.push(text.clone());
            files.insert(path, text.into_bytes());
        }
    }
    for (path, content) in overrides {
        if !files.contains_key(&path) {
            sources.push(content.clone());
            files.insert(path, content.into_bytes());
        }
    }

    let project_paths: HashSet<String> = files.keys().cloned().collect();
    let source_texts: Vec<&str> = sources.iter().map(String::as_str).collect();
    let uploaded = crate::drive::file_data::account_files(state, &project.owner_id, &source_texts, &project_paths).await;
    files.extend(uploaded);

    ProjectInput {
        entrypoint: project.entrypoint.clone(),
        files,
        packages: crate::packages::visibility::load_files_for(state, &project.owner_id).await,
    }
}

#[derive(serde::Deserialize)]
pub struct ListProjectsQuery {
    pub folder_id: Option<String>,
}

pub async fn list_projects(
    Query(query): Query<ListProjectsQuery>,
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<Project>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let projects = if let Some(folder_id) = query.folder_id {
        require_folder_access(&state, &folder_id, &user_id).await?;
        sqlx::query_as::<_, Project>(
            sql("SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at, CAST((SELECT COALESCE(SUM(LENGTH(f.content)), 0) FROM project_files f WHERE f.project_id = projects.id) AS BIGINT) AS size_bytes FROM projects WHERE folder_id = $1 AND owner_id = (SELECT f.owner_id FROM folders f WHERE f.id = $2) ORDER BY updated_at DESC")
        )
        .bind(&folder_id)
        .bind(&folder_id)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, Project>(
            sql("SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at, CAST((SELECT COALESCE(SUM(LENGTH(f.content)), 0) FROM project_files f WHERE f.project_id = projects.id) AS BIGINT) AS size_bytes FROM projects WHERE owner_id = $1 AND folder_id IS NULL ORDER BY updated_at DESC")
        )
        .bind(&user_id)
        .fetch_all(&state.db)
        .await
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(projects))
}

pub async fn list_shared_projects(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<Project>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let projects = sqlx::query_as::<_, Project>(
        sql("SELECT p.id, p.owner_id, p.folder_id, p.name, p.entrypoint, p.thumbnail_svg, \
         p.public_role, p.created_at, p.updated_at, c.role as effective_role \
         FROM projects p \
         INNER JOIN project_collaborators c ON c.project_id = p.id AND c.user_id = $1 \
         ORDER BY p.updated_at DESC")
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(projects))
}

pub async fn create_project(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<Json<Project>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;
    require_owned_folder(&state, &payload.folder_id, &user_id).await?;

    let project_id = Uuid::new_v4().to_string();

    sqlx::query(sql("INSERT INTO projects (id, owner_id, folder_id, name, entrypoint) VALUES ($1, $2, $3, $4, 'main.typ')"))
    .bind(&project_id)
    .bind(&user_id)
    .bind(&payload.folder_id)
    .bind(&payload.name)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let project = sqlx::query_as::<_, Project>(sql("SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at FROM projects WHERE id = $1"))
    .bind(&project_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let seeds = [
        ("typst.toml", default_manifest(&slugify(&payload.name))),
        ("main.typ", "= New Project\n\nStart writing here.\n".to_string()),
    ];
    for (path, content) in seeds {
        let _ = sqlx::query(
            sql("INSERT INTO project_files (id, project_id, path, kind, content, mime_type) VALUES ($1, $2, $3, 'text', $4, 'text/plain')")
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&project_id)
        .bind(path)
        .bind(encode_text_blob(&content))
        .execute(&state.db)
        .await;
    }

    Ok(Json(project))
}

pub async fn get_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<Project>, (StatusCode, String)> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());

    let (mut project, role) = project_role(&state, &id, &user_id_opt)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    project.effective_role = Some(role);
    Ok(Json(project))
}

pub async fn update_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    Json(payload): Json<UpdateProjectRequest>,
) -> Result<Json<Project>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let mut project = sqlx::query_as::<_, Project>(
        sql("SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at FROM projects WHERE id = $1 AND owner_id = $2")
    )
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Project not found".to_string()))?;

    if let Some(name) = payload.name {
        project.name = name;
    }
    if let Some(entrypoint) = payload.entrypoint {
        project.entrypoint = clean_project_path(&entrypoint)?;
    }
    if let Some(folder_id) = payload.folder_id {
        require_owned_folder(&state, &Some(folder_id.clone()), &user_id).await?;
        project.folder_id = if folder_id.is_empty() { None } else { Some(folder_id) };
    }
    if let Some(public_role) = payload.public_role {
        project.public_role = crate::sharing::links::parse_public_role(&public_role, false)?;
    }

    sqlx::query(sql("UPDATE projects SET name = $1, entrypoint = $2, folder_id = $3, public_role = $4, updated_at = $5 WHERE id = $6 AND owner_id = $7"))
    .bind(&project.name)
    .bind(&project.entrypoint)
    .bind(&project.folder_id)
    .bind(&project.public_role)
    .bind(crate::db::timestamp_now())
    .bind(&id)
    .bind(&user_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let project = sqlx::query_as::<_, Project>(sql("SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at FROM projects WHERE id = $1 AND owner_id = $2"))
    .bind(&id)
    .bind(&user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::sharing::access::revalidate_resource(&state, crate::sharing::access::Resource::Project(id.clone())).await;
    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(project))
}

pub async fn delete_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let owned_project = sqlx::query_as::<_, (String,)>(sql("SELECT id FROM projects WHERE id = $1 AND owner_id = $2"))
        .bind(&id)
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if owned_project.is_none() {
        return Err((StatusCode::NOT_FOUND, "Project not found or unauthorized".to_string()));
    }

    sqlx::query(sql("DELETE FROM projects WHERE id = $1 AND owner_id = $2"))
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::sharing::access::revalidate_resource(&state, crate::sharing::access::Resource::Project(id.clone())).await;
    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_project_files(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<ProjectFile>>, (StatusCode, String)> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());
    project_role(&state, &id, &user_id_opt)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let files = sqlx::query_as::<_, ProjectFile>(
        sql("SELECT id, project_id, path, kind, mime_type, created_at FROM project_files WHERE project_id = $1 ORDER BY path ASC")
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(files))
}

pub async fn create_project_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    Json(payload): Json<CreateProjectFileRequest>,
) -> Result<Json<ProjectFile>, (StatusCode, String)> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());
    let (project, role) = project_role(&state, &id, &user_id_opt)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if !can_edit_role(&role) {
        return Err(read_only());
    }

    let path = clean_project_path(&payload.path)?;
    let content = payload.content.unwrap_or_default();
    crate::accounts::storage::ensure_room(&state, &project.owner_id, content.len() as i64).await?;
    let file_id = Uuid::new_v4().to_string();

    sqlx::query(sql("INSERT INTO project_files (id, project_id, path, kind, content, mime_type) VALUES ($1, $2, $3, 'text', $4, 'text/plain')"))
    .bind(&file_id)
    .bind(&id)
    .bind(&path)
    .bind(encode_text_blob(&content))
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let file = sqlx::query_as::<_, ProjectFile>(sql("SELECT id, project_id, path, kind, mime_type, created_at FROM project_files WHERE id = $1"))
    .bind(&file_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    notify_devices(&state, &project.owner_id, DeviceEvent::project(&id)).await;

    crate::drive::activity::mark_project_edited(&state, &id, user_id_opt.as_deref()).await;
    Ok(Json(file))
}

pub async fn upload_project_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());
    let (project, role) = project_role(&state, &id, &user_id_opt)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if !can_edit_role(&role) {
        return Err(read_only());
    }

    let mut uploaded = vec![];

    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        let path = clean_project_path(field.file_name().unwrap_or("unnamed"))?;
        let mime_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data = field.bytes().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?.to_vec();

        let (kind, content) = if is_text_path(&path) {
            let text = String::from_utf8_lossy(&data).to_string();
            ("text", encode_text_blob(&text))
        } else {
            ("binary", data)
        };

        let existing_size = crate::accounts::storage::project_file_size(&state, &id, &path).await;
        crate::accounts::storage::ensure_room(&state, &project.owner_id, content.len() as i64 - existing_size).await?;

        let _ = sqlx::query(
            sql("INSERT INTO project_files (id, project_id, path, kind, content, mime_type) VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (project_id, path) DO UPDATE SET content = excluded.content, kind = excluded.kind, mime_type = excluded.mime_type")
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&id)
        .bind(&path)
        .bind(kind)
        .bind(content)
        .bind(&mime_type)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        uploaded.push(path);
    }

    notify_devices(&state, &project.owner_id, DeviceEvent::project(&id)).await;

    crate::drive::activity::mark_project_edited(&state, &id, user_id_opt.as_deref()).await;
    Ok(Json(serde_json::json!({ "files": uploaded })))
}

pub async fn get_project_file(
    State(state): State<AppState>,
    Path((id, file_id)): Path<(String, String)>,
    jar: SignedCookieJar,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());
    project_role(&state, &id, &user_id_opt)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let file = sqlx::query_as::<_, (String, String, Option<Vec<u8>>)>(
        sql("SELECT kind, mime_type, content FROM project_files WHERE id = $1 AND project_id = $2")
    )
    .bind(&file_id)
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    let (kind, mime_type, content) = file;
    let bytes = content.unwrap_or_default();

    if kind == "binary" {
        Ok(crate::user_content::response(&mime_type, bytes))
    } else {
        Ok(crate::user_content::text_response(decode_text_blob(&bytes)))
    }
}

pub async fn update_project_file(
    State(state): State<AppState>,
    Path((id, file_id)): Path<(String, String)>,
    jar: SignedCookieJar,
    Json(payload): Json<UpdateProjectFileRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());
    let (project, role) = project_role(&state, &id, &user_id_opt)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if !can_edit_role(&role) {
        return Err(read_only());
    }

    let path = clean_project_path(&payload.path)?;
    let result = sqlx::query(sql("UPDATE project_files SET path = $1 WHERE id = $2 AND project_id = $3"))
        .bind(&path)
        .bind(&file_id)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    notify_devices(&state, &project.owner_id, DeviceEvent::project(&id)).await;

    crate::drive::activity::mark_project_edited(&state, &id, user_id_opt.as_deref()).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_project_file(
    State(state): State<AppState>,
    Path((id, file_id)): Path<(String, String)>,
    jar: SignedCookieJar,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());
    let (project, role) = project_role(&state, &id, &user_id_opt)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if !can_edit_role(&role) {
        return Err(read_only());
    }

    let result = sqlx::query(sql("DELETE FROM project_files WHERE id = $1 AND project_id = $2"))
        .bind(&file_id)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    notify_devices(&state, &project.owner_id, DeviceEvent::project(&id)).await;

    crate::drive::activity::mark_project_edited(&state, &id, user_id_opt.as_deref()).await;
    Ok(StatusCode::NO_CONTENT)
}

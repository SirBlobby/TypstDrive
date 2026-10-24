use super::*;
use crate::db::sql;

#[derive(Serialize)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub entrypoint: String,
    pub folder_id: Option<String>,
    pub role: String,
    pub updated_at: String,
}

pub async fn list_projects(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ProjectSummary>>, (StatusCode, String)> {
    let user_id = readable_space(&state, &headers).await?;

    let owned = sqlx::query_as::<_, (String, String, String, Option<String>, String)>(
        sql("SELECT id, name, entrypoint, folder_id, updated_at FROM projects WHERE owner_id = $1 ORDER BY updated_at DESC"),
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let shared = sqlx::query_as::<_, (String, String, String, Option<String>, String, String)>(
        sql("SELECT p.id, p.name, p.entrypoint, p.folder_id, p.updated_at, c.role FROM projects p \
         INNER JOIN project_collaborators c ON c.project_id = p.id AND c.user_id = $1 \
         ORDER BY p.updated_at DESC"),
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut projects: Vec<ProjectSummary> = owned
        .into_iter()
        .map(|(id, name, entrypoint, folder_id, updated_at)| ProjectSummary {
            id,
            name,
            entrypoint,
            folder_id,
            role: "owner".to_string(),
            updated_at,
        })
        .collect();

    projects.extend(
        shared
            .into_iter()
            .map(|(id, name, entrypoint, folder_id, updated_at, role)| ProjectSummary {
                id,
                name,
                entrypoint,
                folder_id,
                role,
                updated_at,
            }),
    );

    Ok(Json(projects))
}

#[derive(Deserialize)]
pub struct CreateProjectBody {
    pub name: String,
    pub entrypoint: Option<String>,
    pub folder_id: Option<String>,
}

pub async fn create_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateProjectBody>,
) -> Result<Json<ProjectSummary>, (StatusCode, String)> {
    let user_id = writable_space(&state, &headers).await?;

    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }

    if let Some(folder_id) = &payload.folder_id {
        owned_folder(&state, folder_id, &user_id).await?;
    }

    let project_id = Uuid::new_v4().to_string();
    let entrypoint = match payload.entrypoint.as_deref() {
        Some(path) => crate::drive::project_paths::clean_project_path(path)?,
        None => "main.typ".to_string(),
    };

    sqlx::query(sql("INSERT INTO projects (id, owner_id, folder_id, name, entrypoint) VALUES ($1, $2, $3, $4, $5)"))
    .bind(&project_id)
    .bind(&user_id)
    .bind(&payload.folder_id)
    .bind(payload.name.trim())
    .bind(&entrypoint)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let project = sqlx::query_as::<_, Project>(sql("SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at FROM projects WHERE id = $1"))
    .bind(&project_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(ProjectSummary {
        id: project.id,
        name: project.name,
        entrypoint: project.entrypoint,
        folder_id: project.folder_id,
        role: "owner".to_string(),
        updated_at: project.updated_at,
    }))
}

pub async fn delete_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = writable_space(&state, &headers).await?;

    let (_, role) = project_with_role(&state, &project_id, &user_id).await?;
    if role != AccessRole::Owner {
        return Err((StatusCode::FORBIDDEN, "Only the owner can delete this project".to_string()));
    }

    sqlx::query(sql("DELETE FROM projects WHERE id = $1 AND owner_id = $2"))
        .bind(&project_id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::sharing::access::revalidate_resource(&state, crate::sharing::access::Resource::Project(project_id.clone())).await;
    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct MoveProjectBody {
    pub folder_id: Option<String>,
}

pub async fn move_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    Json(payload): Json<MoveProjectBody>,
) -> Result<Json<ProjectSummary>, (StatusCode, String)> {
    let user_id = writable_space(&state, &headers).await?;

    if let Some(folder_id) = &payload.folder_id {
        owned_folder(&state, folder_id, &user_id).await?;
    }

    sqlx::query(sql("UPDATE projects SET folder_id = $1 WHERE id = $2 AND owner_id = $3"))
    .bind(&payload.folder_id)
    .bind(&project_id)
    .bind(&user_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let row = sqlx::query_as::<_, (String, String, String, Option<String>, String)>(sql("SELECT id, name, entrypoint, folder_id, updated_at FROM projects WHERE id = $1 AND owner_id = $2"))
    .bind(&project_id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Project not found".to_string()))?;

    notify_devices(&state, &user_id, DeviceEvent::structure()).await;

    Ok(Json(ProjectSummary {
        id: row.0,
        name: row.1,
        entrypoint: row.2,
        folder_id: row.3,
        role: "owner".to_string(),
        updated_at: row.4,
    }))
}

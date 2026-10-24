use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;

use crate::{
    db::sql,
    desktop::devices::{notify_devices, DeviceEvent},
    drive::{
        project_paths::clean_project_path,
        projects::{editable_project, project_entrypoint_after_removal},
    },
    models::{ProjectEntrypoint, ProjectFolderQuery, RenameProjectFolderRequest},
    AppState,
};

type ApiError = (StatusCode, String);

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

async fn project_file_paths(state: &AppState, project_id: &str) -> Result<Vec<(String, String)>, ApiError> {
    sqlx::query_as::<_, (String, String)>(sql("SELECT id, path FROM project_files WHERE project_id = $1"))
        .bind(project_id)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)
}

pub async fn rename_project_folder(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    Json(payload): Json<RenameProjectFolderRequest>,
) -> Result<Json<ProjectEntrypoint>, ApiError> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());
    let project = editable_project(&state, &id, &user_id_opt).await?;
    let from = clean_project_path(&payload.from)?;
    let to = clean_project_path(&payload.to)?;
    let from_prefix = format!("{from}/");
    let to_prefix = format!("{to}/");

    if to == from {
        return Ok(Json(ProjectEntrypoint { entrypoint: project.entrypoint }));
    }
    if to.starts_with(&from_prefix) {
        return Err((StatusCode::BAD_REQUEST, "A folder cannot be moved inside itself".to_string()));
    }

    let files = project_file_paths(&state, &id).await?;
    let moving: Vec<&(String, String)> = files.iter().filter(|(_, path)| path.starts_with(&from_prefix)).collect();
    if moving.is_empty() {
        return Err((StatusCode::NOT_FOUND, "Folder not found".to_string()));
    }
    if files.iter().any(|(_, path)| path == &to || path.starts_with(&to_prefix)) {
        return Err((StatusCode::CONFLICT, "A file or folder with that name already exists".to_string()));
    }

    let mut transaction = state.db.begin().await.map_err(internal_error)?;
    for (file_id, path) in moving {
        let new_path = clean_project_path(&format!("{to_prefix}{}", &path[from_prefix.len()..]))?;
        sqlx::query(sql("UPDATE project_files SET path = $1 WHERE id = $2 AND project_id = $3"))
            .bind(&new_path)
            .bind(file_id)
            .bind(&id)
            .execute(&mut *transaction)
            .await
            .map_err(internal_error)?;
    }

    let entrypoint = match project.entrypoint.strip_prefix(&from_prefix) {
        Some(rest) => format!("{to_prefix}{rest}"),
        None => project.entrypoint.clone(),
    };
    if entrypoint != project.entrypoint {
        sqlx::query(sql("UPDATE projects SET entrypoint = $1 WHERE id = $2"))
            .bind(&entrypoint)
            .bind(&id)
            .execute(&mut *transaction)
            .await
            .map_err(internal_error)?;
    }
    transaction.commit().await.map_err(internal_error)?;

    notify_devices(&state, &project.owner_id, DeviceEvent::project(&id)).await;
    crate::drive::activity::mark_project_edited(&state, &id, user_id_opt.as_deref()).await;
    Ok(Json(ProjectEntrypoint { entrypoint }))
}

pub async fn delete_project_folder(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    Query(query): Query<ProjectFolderQuery>,
) -> Result<Json<ProjectEntrypoint>, ApiError> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());
    let project = editable_project(&state, &id, &user_id_opt).await?;
    let folder = clean_project_path(&query.path)?;
    let prefix = format!("{folder}/");

    let files = project_file_paths(&state, &id).await?;
    let (removed, remaining): (Vec<_>, Vec<_>) = files.into_iter().partition(|(_, path)| path.starts_with(&prefix));
    if removed.is_empty() {
        return Err((StatusCode::NOT_FOUND, "Folder not found".to_string()));
    }

    let mut transaction = state.db.begin().await.map_err(internal_error)?;
    for (file_id, _) in &removed {
        sqlx::query(sql("DELETE FROM project_files WHERE id = $1 AND project_id = $2"))
            .bind(file_id)
            .bind(&id)
            .execute(&mut *transaction)
            .await
            .map_err(internal_error)?;
    }

    let remaining_paths: Vec<String> = remaining.into_iter().map(|(_, path)| path).collect();
    let entrypoint = if project.entrypoint.starts_with(&prefix) {
        project_entrypoint_after_removal(&project.entrypoint, &remaining_paths)
    } else {
        project.entrypoint.clone()
    };
    if entrypoint != project.entrypoint {
        sqlx::query(sql("UPDATE projects SET entrypoint = $1 WHERE id = $2"))
            .bind(&entrypoint)
            .bind(&id)
            .execute(&mut *transaction)
            .await
            .map_err(internal_error)?;
    }
    transaction.commit().await.map_err(internal_error)?;

    notify_devices(&state, &project.owner_id, DeviceEvent::project(&id)).await;
    crate::drive::activity::mark_project_edited(&state, &id, user_id_opt.as_deref()).await;
    Ok(Json(ProjectEntrypoint { entrypoint }))
}

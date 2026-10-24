use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;

use crate::{
    db::sql,
    desktop::devices::{notify_devices, DeviceEvent},
    drive::{project_paths::clean_project_path, projects::editable_project},
    models::ProjectEntrypoint,
    AppState,
};

type ApiError = (StatusCode, String);

fn internal_error(error: sqlx::Error) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

pub async fn set_project_entrypoint(
    State(state): State<AppState>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    Json(payload): Json<ProjectEntrypoint>,
) -> Result<Json<ProjectEntrypoint>, ApiError> {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());
    let project = editable_project(&state, &id, &user_id_opt).await?;
    let entrypoint = clean_project_path(&payload.entrypoint)?;

    let lower = entrypoint.to_lowercase();
    if !lower.ends_with(".typ") && !lower.ends_with(".tex") {
        return Err((StatusCode::BAD_REQUEST, "The entrypoint must be a .typ or .tex file".to_string()));
    }

    let exists = sqlx::query_as::<_, (String,)>(sql("SELECT id FROM project_files WHERE project_id = $1 AND path = $2"))
        .bind(&id)
        .bind(&entrypoint)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .is_some();
    if !exists {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    sqlx::query(sql("UPDATE projects SET entrypoint = $1, updated_at = $2 WHERE id = $3"))
        .bind(&entrypoint)
        .bind(crate::db::timestamp_now())
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    notify_devices(&state, &project.owner_id, DeviceEvent::project(&id)).await;
    crate::drive::activity::mark_project_edited(&state, &id, user_id_opt.as_deref()).await;
    Ok(Json(ProjectEntrypoint { entrypoint }))
}

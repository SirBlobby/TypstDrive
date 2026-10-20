use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    models::{Package, PackageVersion, Project, PublishPackageRequest},
    packages::policy::{is_admin, load_policy},
    packages::visibility,
    drive::projects::decode_text_blob,
    AppState,
};
use crate::db::sql;

#[derive(Deserialize)]
struct Manifest {
    package: PackageMeta,
}

#[derive(Deserialize)]
struct PackageMeta {
    name: String,
    version: String,
    entrypoint: Option<String>,
    description: Option<String>,
}

fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

fn is_valid_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    parts.len() == 3 && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

pub async fn publish_package(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<PublishPackageRequest>,
) -> Result<Json<Package>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let user_is_admin = is_admin(&state, &user_id).await?;
    let policy = load_policy(&state).await;
    policy.check_can_publish(user_is_admin)?;

    let project = sqlx::query_as::<_, Project>(
        sql("SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at FROM projects WHERE id = $1 AND owner_id = $2")
    )
    .bind(&payload.project_id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Project not found".to_string()))?;

    let files = sqlx::query_as::<_, (String, String, Option<Vec<u8>>)>(
        sql("SELECT path, kind, content FROM project_files WHERE project_id = $1")
    )
    .bind(&project.id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut snapshot: Vec<(String, Vec<u8>)> = Vec::new();
    let mut manifest_text: Option<String> = None;
    for (path, kind, content) in files {
        let bytes = if kind == "binary" {
            content.unwrap_or_default()
        } else {
            decode_text_blob(&content.unwrap_or_default()).into_bytes()
        };
        if path == "typst.toml" {
            manifest_text = Some(String::from_utf8_lossy(&bytes).to_string());
        }
        snapshot.push((path, bytes));
    }

    let package_size: i64 = snapshot.iter().map(|(_, bytes)| bytes.len() as i64).sum();
    policy.check_package_size(package_size)?;

    let manifest_text = manifest_text
        .ok_or((StatusCode::BAD_REQUEST, "Project has no typst.toml manifest".to_string()))?;
    let manifest: Manifest = toml::from_str(&manifest_text)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid typst.toml: {}", e)))?;

    let name = manifest.package.name.trim().to_string();
    let version = payload.version.unwrap_or(manifest.package.version).trim().to_string();
    let entrypoint = manifest.package.entrypoint.unwrap_or_else(|| "lib.typ".to_string());

    if !is_valid_name(&name) {
        return Err((StatusCode::BAD_REQUEST, "Invalid package name (lowercase letters, digits, '-' and '_' only)".to_string()));
    }
    if !is_valid_version(&version) {
        return Err((StatusCode::BAD_REQUEST, "Version must be in the form major.minor.patch".to_string()));
    }

    let existing = visibility::find_by_name(&state, &name).await?;

    let package = match existing {
        Some(pkg) => {
            if pkg.owner_id != user_id {
                return Err((StatusCode::FORBIDDEN, "A package with this name is owned by another user".to_string()));
            }
            pkg
        }
        None => {
            let package_visibility = policy.visibility_for_new_package(payload.visibility.as_deref(), user_is_admin)?;
            let package_id = Uuid::new_v4().to_string();
            sqlx::query(sql("INSERT INTO packages (id, owner_id, namespace, name, description, visibility) VALUES ($1, $2, 'typstdrive', $3, $4, $5)"))
            .bind(&package_id)
            .bind(&user_id)
            .bind(&name)
            .bind(&manifest.package.description)
            .bind(package_visibility)
            .execute(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            sqlx::query_as::<_, Package>(sql("SELECT id, owner_id, namespace, name, description, visibility, created_at FROM packages WHERE id = $1"))
            .bind(&package_id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        }
    };

    let version_exists = sqlx::query_as::<_, (String,)>(
        sql("SELECT id FROM package_versions WHERE package_id = $1 AND version = $2")
    )
    .bind(&package.id)
    .bind(&version)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if version_exists.is_some() {
        return Err((StatusCode::CONFLICT, format!("Version {} already published; versions are immutable", version)));
    }

    let (version_count,) = sqlx::query_as::<_, (i64,)>(
        sql("SELECT CAST(COUNT(*) AS BIGINT) FROM package_versions WHERE package_id = $1")
    )
    .bind(&package.id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    policy.check_version_count(version_count)?;

    let version_id = Uuid::new_v4().to_string();
    sqlx::query(
        sql("INSERT INTO package_versions (id, package_id, version, entrypoint, manifest) VALUES ($1, $2, $3, $4, $5)")
    )
    .bind(&version_id)
    .bind(&package.id)
    .bind(&version)
    .bind(&entrypoint)
    .bind(manifest_text.into_bytes())
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for (path, data) in snapshot {
        let _ = sqlx::query(
            sql("INSERT INTO package_files (id, version_id, path, data) VALUES ($1, $2, $3, $4)")
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&version_id)
        .bind(&path)
        .bind(&data)
        .execute(&state.db)
        .await;
    }

    Ok(Json(package))
}

pub async fn list_packages(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<Package>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let packages = if is_admin(&state, &user_id).await? {
        visibility::list_all(&state).await?
    } else {
        visibility::list_visible(&state, &user_id).await?
    };

    Ok(Json(packages))
}

pub async fn list_versions(
    State(state): State<AppState>,
    Path(name): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<PackageVersion>>, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let not_found = (StatusCode::NOT_FOUND, "Package not found".to_string());
    let package = visibility::find_by_name(&state, &name).await?.ok_or(not_found.clone())?;
    if !is_admin(&state, &user_id).await? && !visibility::can_view(&state, &package.id, &user_id).await {
        return Err(not_found);
    }

    let versions = sqlx::query_as::<_, PackageVersion>(
        sql("SELECT id, package_id, version, entrypoint, created_at \
         FROM package_versions WHERE package_id = $1 ORDER BY created_at DESC")
    )
    .bind(&package.id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(versions))
}

pub async fn delete_package(
    State(state): State<AppState>,
    Path(name): Path<String>,
    jar: SignedCookieJar,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let user_is_admin = is_admin(&state, &user_id).await?;
    if !user_is_admin && !load_policy(&state).await.allow_owner_delete {
        return Err((StatusCode::FORBIDDEN, "Only administrators can delete packages".to_string()));
    }

    let result = if user_is_admin {
        sqlx::query(sql("DELETE FROM packages WHERE namespace = 'typstdrive' AND name = $1"))
            .bind(&name)
            .execute(&state.db)
            .await
    } else {
        sqlx::query(sql("DELETE FROM packages WHERE namespace = 'typstdrive' AND name = $1 AND owner_id = $2"))
            .bind(&name)
            .bind(&user_id)
            .execute(&state.db)
            .await
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Package not found or unauthorized".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

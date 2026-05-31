use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    models::{Package, PackageVersion, PublishPackageRequest, Space},
    spaces::decode_text_blob,
    AppState,
};

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

    let space = sqlx::query_as::<_, Space>(
        "SELECT id, owner_id, folder_id, name, entrypoint, thumbnail_svg, public_role, created_at, updated_at FROM spaces WHERE id = ? AND owner_id = ?"
    )
    .bind(&payload.space_id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Space not found".to_string()))?;

    let files = sqlx::query_as::<_, (String, String, Option<Vec<u8>>)>(
        "SELECT path, kind, content FROM space_files WHERE space_id = ?"
    )
    .bind(&space.id)
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

    let manifest_text = manifest_text
        .ok_or((StatusCode::BAD_REQUEST, "Space has no typst.toml manifest".to_string()))?;
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

    let existing = sqlx::query_as::<_, Package>(
        "SELECT id, owner_id, namespace, name, description, created_at FROM packages WHERE namespace = 'typstdrive' AND name = ?"
    )
    .bind(&name)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let package = match existing {
        Some(pkg) => {
            if pkg.owner_id != user_id {
                return Err((StatusCode::FORBIDDEN, "A package with this name is owned by another user".to_string()));
            }
            pkg
        }
        None => {
            let package_id = Uuid::new_v4().to_string();
            sqlx::query_as::<_, Package>(
                "INSERT INTO packages (id, owner_id, namespace, name, description) VALUES (?, ?, 'typstdrive', ?, ?) RETURNING id, owner_id, namespace, name, description, created_at"
            )
            .bind(&package_id)
            .bind(&user_id)
            .bind(&name)
            .bind(&manifest.package.description)
            .fetch_one(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        }
    };

    let version_exists = sqlx::query_as::<_, (String,)>(
        "SELECT id FROM package_versions WHERE package_id = ? AND version = ?"
    )
    .bind(&package.id)
    .bind(&version)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if version_exists.is_some() {
        return Err((StatusCode::CONFLICT, format!("Version {} already published; versions are immutable", version)));
    }

    let version_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO package_versions (id, package_id, version, entrypoint, manifest) VALUES (?, ?, ?, ?, ?)"
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
            "INSERT INTO package_files (id, version_id, path, data) VALUES (?, ?, ?, ?)"
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
    jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let packages = sqlx::query_as::<_, Package>(
        "SELECT p.id, p.owner_id, p.namespace, p.name, p.description, p.created_at, \
         u.username as owner_name, \
         (SELECT v.version FROM package_versions v WHERE v.package_id = p.id ORDER BY v.created_at DESC LIMIT 1) as latest_version \
         FROM packages p JOIN users u ON u.id = p.owner_id \
         ORDER BY p.name ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(packages))
}

pub async fn list_versions(
    State(state): State<AppState>,
    Path(name): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<PackageVersion>>, (StatusCode, String)> {
    jar.get("session_user_id").map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    let versions = sqlx::query_as::<_, PackageVersion>(
        "SELECT v.id, v.package_id, v.version, v.entrypoint, v.created_at \
         FROM package_versions v JOIN packages p ON p.id = v.package_id \
         WHERE p.namespace = 'typstdrive' AND p.name = ? ORDER BY v.created_at DESC"
    )
    .bind(&name)
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

    let is_admin = sqlx::query_as::<_, (i64,)>("SELECT is_admin FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(|(a,)| a != 0)
        .unwrap_or(false);

    let result = if is_admin {
        sqlx::query("DELETE FROM packages WHERE namespace = 'typstdrive' AND name = ?")
            .bind(&name)
            .execute(&state.db)
            .await
    } else {
        sqlx::query("DELETE FROM packages WHERE namespace = 'typstdrive' AND name = ? AND owner_id = ?")
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

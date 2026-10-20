use std::io::Cursor;

use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
};
use axum_extra::extract::cookie::SignedCookieJar;
use image::codecs::webp::WebPEncoder;
use image::{ImageReader, Limits};

use crate::AppState;
use crate::db::sql;

pub const MAX_UPLOAD_BYTES: usize = 15 * 1024 * 1024;

const MAX_AVATAR_DIMENSION: u32 = 512;
const MAX_SOURCE_DIMENSION: u32 = 8192;
const AVATAR_MIME_TYPE: &str = "image/webp";

fn session_user_id(jar: &SignedCookieJar) -> Result<String, (StatusCode, String)> {
    jar.get("session_user_id")
        .map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))
}

pub fn compress_image(source: &[u8], max_dimension: u32) -> Result<Vec<u8>, String> {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SOURCE_DIMENSION);
    limits.max_image_height = Some(MAX_SOURCE_DIMENSION);

    let mut reader = ImageReader::new(Cursor::new(source))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    reader.limits(limits);

    let mut image = reader
        .decode()
        .map_err(|_| "Unsupported or corrupt image. Use PNG, JPEG, GIF or WebP.".to_string())?;

    if image.width() > max_dimension || image.height() > max_dimension {
        image = image.thumbnail(max_dimension, max_dimension);
    }

    let pixels = image.to_rgba8();
    let mut encoded = Vec::new();
    pixels
        .write_with_encoder(WebPEncoder::new_lossless(&mut encoded))
        .map_err(|e| e.to_string())?;

    Ok(encoded)
}

pub async fn avatar_updated_at(state: &AppState, user_id: &str) -> Option<String> {
    sqlx::query_as::<_, (String,)>(sql("SELECT updated_at FROM user_avatars WHERE user_id = $1"))
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .map(|(updated_at,)| updated_at)
}

pub async fn get_avatar(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = session_user_id(&jar)?;
    avatar_response(&state, &user_id).await
}

pub async fn get_user_avatar(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    session_user_id(&jar)?;
    avatar_response(&state, &user_id).await
}

async fn avatar_response(state: &AppState, user_id: &str) -> Result<impl IntoResponse, (StatusCode, String)> {
    let avatar = sqlx::query_as::<_, (Vec<u8>,)>(sql("SELECT data FROM user_avatars WHERE user_id = $1"))
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match avatar {
        Some((data,)) => Ok((
            [
                (header::CONTENT_TYPE, AVATAR_MIME_TYPE),
                (header::CACHE_CONTROL, "private, max-age=31536000, immutable"),
            ],
            data,
        )),
        None => Err((StatusCode::NOT_FOUND, "No profile picture".to_string())),
    }
}

pub async fn upload_avatar(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    mut multipart: Multipart,
) -> Result<String, (StatusCode, String)> {
    let user_id = session_user_id(&jar)?;

    let field = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
        .ok_or((StatusCode::BAD_REQUEST, "No image provided".to_string()))?;
    let source = field
        .bytes()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
        .to_vec();

    let compressed = tokio::task::spawn_blocking(move || compress_image(&source, MAX_AVATAR_DIMENSION))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let updated_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();

    sqlx::query(sql("DELETE FROM user_avatars WHERE user_id = $1"))
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query(sql("INSERT INTO user_avatars (user_id, data, updated_at) VALUES ($1, $2, $3)"))
        .bind(&user_id)
        .bind(&compressed)
        .bind(&updated_at)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(updated_at)
}

pub async fn delete_avatar(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = session_user_id(&jar)?;

    sqlx::query(sql("DELETE FROM user_avatars WHERE user_id = $1"))
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

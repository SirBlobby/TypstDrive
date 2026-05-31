use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{api_keys::hash_key, compiler::ProjectInput, AppState};

#[derive(Deserialize)]
pub struct RenderRequest {
    pub code: String,
    pub format: String,
    pub files: Option<Vec<InlineFile>>,
}

#[derive(Deserialize)]
pub struct InlineFile {
    pub name: String,
    pub data: String, // base64-encoded
}

#[derive(Serialize)]
struct CompileErrorDetail {
    message: String,
    severity: String,
    line: Option<usize>,
    column: Option<usize>,
}

#[derive(Serialize)]
struct CompileErrorResponse {
    error: String,
    details: Vec<CompileErrorDetail>,
}

fn line_and_column(code: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    for (index, character) in code.char_indices() {
        if index >= offset {
            break;
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

fn compute_cache_key(format: &str, code: &str, files: &Option<Vec<InlineFile>>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format.as_bytes());
    hasher.update(b"\x00");
    hasher.update(code.as_bytes());
    if let Some(files) = files {
        let mut pairs: Vec<_> = files.iter().map(|f| (f.name.as_str(), f.data.as_str())).collect();
        pairs.sort_by_key(|(n, _)| *n);
        for (name, data) in pairs {
            hasher.update(b"\x01");
            hasher.update(name.as_bytes());
            hasher.update(data.as_bytes());
        }
    }
    format!("{:x}", hasher.finalize())
}

pub async fn render_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<RenderRequest>,
) -> impl IntoResponse {
    // Extract Bearer token
    let api_key = match headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .filter(|v| v.starts_with("Bearer "))
        .map(|v| v[7..].to_string())
    {
        Some(k) => k,
        None => return (StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header. Use: Authorization: Bearer <api-key>").into_response(),
    };

    if payload.format != "png" && payload.format != "pdf" {
        return (StatusCode::BAD_REQUEST, "Invalid format. Must be 'png' or 'pdf'").into_response();
    }

    if payload.code.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "code cannot be empty").into_response();
    }

    let key_hash = hash_key(&api_key);

    let key_row = sqlx::query_as::<_, (String, String, i64)>(
        "SELECT id, user_id, rate_limit FROM api_keys WHERE key_hash = ?"
    )
    .bind(&key_hash)
    .fetch_optional(&state.db)
    .await;

    let (key_id, user_id, rate_limit) = match key_row {
        Ok(Some(row)) => row,
        Ok(None) => return (StatusCode::UNAUTHORIZED, "Invalid API key").into_response(),
        Err(e) => {
            let msg = format!("Database error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response();
        }
    };

    // Rate limiting: fixed window of 60 seconds
    {
        let mut limiter = state.rate_limiter.lock().await;
        let now = std::time::Instant::now();
        let window = std::time::Duration::from_secs(60);
        let entry = limiter.entry(key_id.clone()).or_insert((0u32, now));
        if now.duration_since(entry.1) > window {
            entry.0 = 1;
            entry.1 = now;
        } else if entry.0 < rate_limit as u32 {
            entry.0 += 1;
        } else {
            return (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded. Max requests per minute reached.").into_response();
        }
    }

    let now = chrono::Utc::now();
    let now_str = now.to_rfc3339();
    let today = now.format("%Y-%m-%d").to_string();

    let _ = sqlx::query("UPDATE api_keys SET last_used_at = ? WHERE id = ?")
        .bind(&now_str)
        .bind(&key_id)
        .execute(&state.db)
        .await;

    let _ = sqlx::query(
        "INSERT INTO api_key_usage (key_id, date, count) VALUES (?, ?, 1) \
         ON CONFLICT (key_id, date) DO UPDATE SET count = api_key_usage.count + 1"
    )
    .bind(&key_id)
    .bind(&today)
    .execute(&state.db)
    .await;

    let minute_str = now.format("%Y-%m-%d %H:%M").to_string();
    let _ = sqlx::query(
        "INSERT INTO api_key_usage_detail (key_id, minute, count) VALUES (?, ?, 1) \
         ON CONFLICT (key_id, minute) DO UPDATE SET count = api_key_usage_detail.count + 1"
    )
    .bind(&key_id)
    .bind(&minute_str)
    .execute(&state.db)
    .await;

    let cutoff_minute = (chrono::Utc::now() - chrono::TimeDelta::hours(25))
        .format("%Y-%m-%d %H:%M")
        .to_string();
    let _ = sqlx::query("DELETE FROM api_key_usage_detail WHERE minute < ?")
        .bind(&cutoff_minute)
        .execute(&state.db)
        .await;

    // Check cache
    let cache_key = compute_cache_key(&payload.format, &payload.code, &payload.files);
    let content_type: &'static str = if payload.format == "pdf" { "application/pdf" } else { "image/png" };

    if let Ok(Some((data, created_at))) = sqlx::query_as::<_, (Vec<u8>, String)>(
        "SELECT data, created_at FROM api_render_cache WHERE content_hash = ? AND format = ?"
    )
    .bind(&cache_key)
    .bind(&payload.format)
    .fetch_optional(&state.db)
    .await
    {
        if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&created_at) {
            let age = chrono::Utc::now().signed_duration_since(parsed.with_timezone(&chrono::Utc));
            if age.num_seconds() < 3600 {
                return (StatusCode::OK, [(header::CONTENT_TYPE, content_type)], data).into_response();
            }
        }
    }

    // Load user's account files
    let mut files_map: HashMap<String, Vec<u8>> = HashMap::new();
    if let Ok(files) = sqlx::query_as::<_, (String, Vec<u8>)>(
        "SELECT name, data FROM files WHERE owner_id = ?"
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    {
        for (name, data) in files {
            files_map.insert(name, data);
        }
    }

    // Inline files override account files
    if let Some(inline_files) = &payload.files {
        for f in inline_files {
            if let Ok(decoded) = BASE64.decode(&f.data) {
                files_map.insert(f.name.clone(), decoded);
            }
        }
    }

    // Compile
    let compiler = state.compiler.lock().await;
    let result = match payload.format.as_str() {
        "pdf" => compiler.export_pdf(ProjectInput::single(payload.code.clone(), files_map)),
        "png" => compiler.export_png(ProjectInput::single(payload.code.clone(), files_map)),
        _ => unreachable!(),
    };
    drop(compiler);

    match result {
        Ok(data) => {
            // Store in cache (ignore errors — concurrent inserts are fine)
            let cache_id = Uuid::new_v4().to_string();
            let _ = sqlx::query(
                "INSERT INTO api_render_cache (id, content_hash, format, data, created_at) VALUES (?, ?, ?, ?, ?)"
            )
            .bind(&cache_id)
            .bind(&cache_key)
            .bind(&payload.format)
            .bind(&data)
            .bind(&now_str)
            .execute(&state.db)
            .await;

            (StatusCode::OK, [(header::CONTENT_TYPE, content_type)], data).into_response()
        }
        Err(diagnostics) => {
            let details: Vec<CompileErrorDetail> = diagnostics
                .into_iter()
                .map(|(diagnostic, range)| {
                    let (line, column) = match range.as_ref() {
                        Some(range) => {
                            let (line, column) = line_and_column(&payload.code, range.start);
                            (Some(line), Some(column))
                        }
                        None => (None, None),
                    };
                    CompileErrorDetail {
                        message: diagnostic.message.to_string(),
                        severity: format!("{:?}", diagnostic.severity).to_lowercase(),
                        line,
                        column,
                    }
                })
                .collect();

            let summary = details
                .iter()
                .map(|detail| match (detail.line, detail.column) {
                    (Some(line), Some(column)) => {
                        format!("{} (line {}, column {})", detail.message, line, column)
                    }
                    _ => detail.message.clone(),
                })
                .collect::<Vec<_>>()
                .join("; ");

            let error = if summary.is_empty() {
                "Typst compilation failed.".to_string()
            } else {
                format!("Typst compilation failed: {}", summary)
            };

            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(CompileErrorResponse { error, details }),
            )
                .into_response()
        }
    }
}

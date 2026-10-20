use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::sharing::access;
use crate::AppState;
use crate::drive::projects::decode_text_blob;
use crate::db::sql;

#[derive(Deserialize)]
pub struct CompileRequest {
    #[serde(default)]
    pub text: Option<String>,
    pub document_id: Option<String>,
    pub project_id: Option<String>,
    #[serde(default)]
    pub files: Option<std::collections::HashMap<String, String>>,
}

use crate::compile::compiler::{DocumentStats, ProjectInput};

fn map_diagnostics(
    diags: Vec<(typst::diag::SourceDiagnostic, Option<std::ops::Range<usize>>)>,
) -> Vec<Diagnostic> {
    diags
        .into_iter()
        .map(|(d, range)| Diagnostic {
            message: d.message.to_string(),
            severity: format!("{:?}", d.severity),
            from: range.as_ref().map(|r| r.start),
            to: range.as_ref().map(|r| r.end),
        })
        .collect()
}

#[derive(Serialize)]
pub struct CompileResponse {
    pub svgs: Option<Vec<String>>,
    pub errors: Option<Vec<Diagnostic>>,
    pub stats: Option<DocumentStats>,
}

#[derive(Serialize)]
pub struct Diagnostic {
    pub message: String,
    pub severity: String,
    pub from: Option<usize>,
    pub to: Option<usize>,
}

fn unauthorized_compile() -> Json<CompileResponse> {
    Json(CompileResponse {
        svgs: None,
        errors: Some(vec![Diagnostic {
            message: "Unauthorized".to_string(),
            severity: "Error".to_string(),
            from: None,
            to: None,
        }]),
        stats: None,
    })
}

async fn document_source(
    state: &AppState,
    payload: &CompileRequest,
    user_id: &Option<String>,
) -> Result<(String, HashMap<String, Vec<u8>>, bool), ()> {
    let submitted_text = payload.text.clone().unwrap_or_default();
    let Some(document_id) = &payload.document_id else {
        if user_id.is_none() {
            return Err(());
        }
        return Ok((submitted_text, HashMap::new(), false));
    };

    let (document, role) = access::document_role(state, document_id, user_id).await.ok_or(())?;
    let files_map = crate::drive::document_assets::compile_files(state, &document, role, user_id).await;
    if role.can_edit() {
        Ok((submitted_text, files_map, true))
    } else {
        let saved_text = decode_text_blob(document.content.as_deref().unwrap_or_default());
        Ok((saved_text, files_map, false))
    }
}

pub async fn compile_handler(
    State(state): State<AppState>,
    jar: axum_extra::extract::cookie::SignedCookieJar,
    Json(payload): Json<CompileRequest>,
) -> impl IntoResponse {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());

    if let Some(project_id) = &payload.project_id {
        let Some((project, role)) = access::project_access(&state, project_id, &user_id_opt).await else {
            return unauthorized_compile();
        };

        let overrides = if role.can_edit() { payload.files.clone().unwrap_or_default() } else { HashMap::new() };
        let input = crate::drive::projects::assemble_project(&state, &project, overrides).await;

        let compiler = state.compiler.lock().await;
        let result = compiler.compile_svg(input);
        drop(compiler);

        return match result {
            Ok((svgs, thumbnail, stats)) => {
                if role.can_edit() {
                    let _ = sqlx::query(sql("UPDATE projects SET thumbnail_svg = $1 WHERE id = $2"))
                        .bind(&thumbnail)
                        .bind(&project.id)
                        .execute(&state.db)
                        .await;
                }
                Json(CompileResponse {
                    svgs: Some(svgs),
                    errors: None,
                    stats: Some(stats),
                })
            }
            Err(diags) => Json(CompileResponse {
                svgs: None,
                errors: Some(map_diagnostics(diags)),
                stats: None,
            }),
        };
    }

    let Ok((source_text, files_map, can_save_thumbnail)) = document_source(&state, &payload, &user_id_opt).await else {
        return unauthorized_compile();
    };

    let compiler = state.compiler.lock().await;
    let result = compiler.compile_svg(ProjectInput::single(source_text, files_map));
    drop(compiler);

    match result {
        Ok((svgs, thumbnail, stats)) => {
            if let (Some(doc_id), true) = (&payload.document_id, can_save_thumbnail) {
                let _ = sqlx::query(sql("UPDATE documents SET thumbnail_svg = $1 WHERE id = $2"))
                    .bind(&thumbnail)
                    .bind(doc_id)
                    .execute(&state.db)
                    .await;
            }

            Json(CompileResponse {
                svgs: Some(svgs),
                errors: None,
                stats: Some(stats),
            })
        }
        Err(diags) => Json(CompileResponse {
            svgs: None,
            errors: Some(map_diagnostics(diags)),
            stats: None,
        }),
    }
}

pub async fn export_handler(
    State(state): State<AppState>,
    jar: axum_extra::extract::cookie::SignedCookieJar,
    Path(format): Path<String>,
    Json(payload): Json<CompileRequest>,
) -> impl IntoResponse {
    let user_id_opt = jar.get("session_user_id").map(|c| c.value().to_string());

    let input = if let Some(project_id) = &payload.project_id {
        match access::project_access(&state, project_id, &user_id_opt).await {
            Some((project, role)) => {
                let overrides = if role.can_edit() { payload.files.clone().unwrap_or_default() } else { HashMap::new() };
                crate::drive::projects::assemble_project(&state, &project, overrides).await
            }
            None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
        }
    } else {
        match document_source(&state, &payload, &user_id_opt).await {
            Ok((source_text, files_map, _)) => ProjectInput::single(source_text, files_map),
            Err(()) => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
        }
    };

    let compiler = state.compiler.lock().await;

    match format.as_str() {
        "pdf" => match compiler.export_pdf(input) {
            Ok(bytes) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/pdf")],
                bytes,
            )
                .into_response(),
            Err(_) => (StatusCode::BAD_REQUEST, "Compilation failed").into_response(),
        },
        "png" => match compiler.export_png(input) {
            Ok(bytes) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "image/png")],
                bytes,
            )
                .into_response(),
            Err(_) => (StatusCode::BAD_REQUEST, "Compilation failed").into_response(),
        },
        "svg" => match compiler.compile_svg(input) {
            Ok((svgs, _, _)) => {
                let mut combined = String::new();
                for svg in svgs {
                    combined.push_str(&svg);
                    combined.push('\n');
                }
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "image/svg+xml")],
                    combined.into_bytes(),
                )
                    .into_response()
            }
            Err(_) => (StatusCode::BAD_REQUEST, "Compilation failed").into_response(),
        },
        _ => (StatusCode::NOT_FOUND, "Format not supported").into_response(),
    }
}

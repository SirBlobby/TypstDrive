use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use yrs_axum::ws::{AxumSink, AxumStream};
use yrs_axum::broadcast::BroadcastGroup;
use yrs::sync::Awareness;
use yrs::{Doc, ReadTxn, Transact, Update};
use yrs::updates::decoder::Decode;
use futures_util::stream::StreamExt;
use crate::AppState;
use crate::models::Document;

#[derive(Deserialize)]
pub struct CompileRequest {
    pub text: String,
    pub document_id: Option<String>,
}

#[derive(Serialize)]
pub struct CompileResponse {
    pub svgs: Option<Vec<String>>,
    pub errors: Option<Vec<Diagnostic>>,
}

#[derive(Serialize)]
pub struct Diagnostic {
    pub message: String,
    pub severity: String,
}

pub async fn yjs_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut bcast_map = state.bcast_map.lock().await;
    let bcast = if let Some(bcast) = bcast_map.get(&id) {
        bcast.clone()
    } else {
        let doc = sqlx::query_as::<_, Document>(
            "SELECT id, owner_id, folder_id, title, content, thumbnail_svg, created_at, updated_at FROM documents WHERE id = ?"
        )
        .bind(&id)
        .fetch_optional(&state.db)
        .await;

        let ydoc = Doc::new();
        
        if let Ok(Some(db_doc)) = doc {
            if let Some(content) = db_doc.content {
                if let Ok(update) = Update::decode_v1(&content) {
                    ydoc.transact_mut().apply_update(update);
                }
            }
        }
        
        let awareness = Arc::new(RwLock::new(Awareness::new(ydoc)));
        let new_bcast = Arc::new(BroadcastGroup::new(awareness.clone(), 10).await);
        bcast_map.insert(id.clone(), new_bcast.clone());

        let save_db = state.db.clone();
        let save_id = id.clone();
        let save_awareness = awareness.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            loop {
                interval.tick().await;
                let doc = save_awareness.read().await;
                let content = doc.doc().transact().encode_state_as_update_v1(&yrs::StateVector::default());
                let _ = sqlx::query("UPDATE documents SET content = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
                    .bind(content)
                    .bind(&save_id)
                    .execute(&save_db)
                    .await;
            }
        });

        new_bcast
    };

    drop(bcast_map);

    ws.on_upgrade(move |socket| async move {
        let (sink, stream) = socket.split();
        let sink = Arc::new(Mutex::new(AxumSink(sink)));
        let stream = AxumStream(stream);
        let sub = bcast.subscribe(sink, stream);
        match sub.completed().await {
            Ok(_) => println!("broadcasting for channel finished successfully"),
            Err(e) => eprintln!("broadcasting for channel finished abruptly: {}", e),
        }
    })
}

pub async fn compile_handler(
    State(state): State<AppState>,
    Json(payload): Json<CompileRequest>,
) -> impl IntoResponse {
    let mut files_map = std::collections::HashMap::new();
    if let Some(doc_id) = &payload.document_id {
        if let Ok(doc) = sqlx::query_as::<_, crate::models::Document>("SELECT id, owner_id, folder_id, title, content, thumbnail_svg, created_at, updated_at FROM documents WHERE id = ?").bind(doc_id).fetch_one(&state.db).await {
            if let Ok(files) = sqlx::query_as::<_, (String, Vec<u8>)>("SELECT name, data FROM files WHERE owner_id = ?")
                .bind(doc.owner_id)
                .fetch_all(&state.db)
                .await
            {
                for (name, data) in files {
                    files_map.insert(name, data);
                }
            }
        }
    }

    let compiler = state.compiler.lock().await;
    match compiler.compile_svg(payload.text, files_map) {
        Ok((svgs, thumbnail)) => {
            if let Some(doc_id) = &payload.document_id {
                let _ = sqlx::query("UPDATE documents SET thumbnail_svg = ? WHERE id = ?")
                    .bind(&thumbnail)
                    .bind(doc_id)
                    .execute(&state.db)
                    .await;
            }
            
            Json(CompileResponse {
                svgs: Some(svgs),
                errors: None,
            })
        }
        Err(diags) => {
            let errors = diags
                .into_iter()
                .map(|d| Diagnostic {
                    message: d.message.to_string(),
                    severity: format!("{:?}", d.severity),
                })
                .collect();
            Json(CompileResponse {
                svgs: None,
                errors: Some(errors),
            })
        }
    }
}

pub async fn export_handler(
    State(state): State<AppState>,
    Path(format): Path<String>,
    Json(payload): Json<CompileRequest>,
) -> impl IntoResponse {
    let mut files_map = std::collections::HashMap::new();
    if let Some(doc_id) = &payload.document_id {
        if let Ok(doc) = sqlx::query_as::<_, crate::models::Document>("SELECT id, owner_id, folder_id, title, content, thumbnail_svg, created_at, updated_at FROM documents WHERE id = ?").bind(doc_id).fetch_one(&state.db).await {
            if let Ok(files) = sqlx::query_as::<_, (String, Vec<u8>)>("SELECT name, data FROM files WHERE owner_id = ?")
                .bind(doc.owner_id)
                .fetch_all(&state.db)
                .await
            {
                for (name, data) in files {
                    files_map.insert(name, data);
                }
            }
        }
    }

    let compiler = state.compiler.lock().await;

    match format.as_str() {
        "pdf" => match compiler.export_pdf(payload.text, files_map.clone()) {
            Ok(bytes) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/pdf")],
                bytes,
            )
                .into_response(),
            Err(_) => (StatusCode::BAD_REQUEST, "Compilation failed").into_response(),
        },
        "png" => match compiler.export_png(payload.text, files_map.clone()) {
            Ok(bytes) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "image/png")],
                bytes,
            )
                .into_response(),
            Err(_) => (StatusCode::BAD_REQUEST, "Compilation failed").into_response(),
        },
        "svg" => match compiler.compile_svg(payload.text, files_map.clone()) {
            Ok((svgs, _)) => {
                
                
                let mut combined = String::new();
                for svg in svgs {
                    combined.push_str(&svg);
                    combined.push_str("\n");
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

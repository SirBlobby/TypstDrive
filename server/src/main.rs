use axum::{
    routing::{get, post, put, delete, patch},
    Router,
};
use axum_extra::extract::cookie::Key;
use sqlx::AnyPool;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use yrs_axum::broadcast::BroadcastGroup;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod compiler;
mod db;
mod docs;
mod folders;
mod files;
mod handlers;
mod models;
mod world;
mod collab;

use compiler::TypstCompiler;
use handlers::{compile_handler, export_handler, yjs_handler};

#[derive(Clone)]
pub struct AppState {
    pub compiler: Arc<Mutex<TypstCompiler>>,
    pub bcast_map: Arc<Mutex<HashMap<String, Arc<BroadcastGroup>>>>,
    pub db: AnyPool,
    pub key: Key,
}

impl axum::extract::FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting TypstDrive Server");

    let db = db::init_db().await;

    
    let key = Key::generate();

    let state = AppState {
        compiler: Arc::new(Mutex::new(TypstCompiler::new())),
        bcast_map: Arc::new(Mutex::new(HashMap::new())),
        db,
        key,
    };

    let api_routes = Router::new()
        .route("/compile", post(compile_handler))
        .route("/export/{format}", post(export_handler))
        .route("/export/pandoc/{format}", post(handlers::pandoc_export_handler))
        .route("/import/pandoc", post(handlers::pandoc_import_handler))
        .route("/lsp/{id}", get(handlers::lsp_handler))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me).put(auth::update_profile))
        .route("/auth/storage", get(auth::storage_stats))
        .route("/auth/change-password", put(auth::change_password))
        .route("/folders", get(folders::list_folders).post(folders::create_folder))
        .route("/folders/{id}", delete(folders::delete_folder).patch(folders::update_folder))
        .route("/fonts", get(files::list_fonts))
        .route("/files", get(files::list_files).post(files::upload_file_global))
        .route("/files/{id}", delete(files::delete_file).patch(files::update_file))
        .route("/files/{id}/data", get(files::get_file_data))
        .route("/docs", get(docs::list_documents).post(docs::create_document))
        .route("/docs/accept-invite", get(collab::accept_invite))
        .route("/docs/{id}", get(docs::get_document).delete(docs::delete_document).patch(docs::update_document))
        .route("/docs/{id}/files", post(docs::upload_file))
        .route("/docs/{id}/invite", post(collab::invite_collaborator))
        .route("/docs/{id}/comments", get(collab::get_comments).post(collab::add_comment))
        .route("/docs/{id}/versions", get(collab::get_versions).post(collab::create_version))
        .route("/comments/{id}", patch(collab::update_comment).delete(collab::delete_comment));

    let yjs_routes = Router::new()
        .route("/{id}", get(yjs_handler));

    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "../build".to_string());

    let app = Router::new()
        .nest("/api", api_routes.layer(TraceLayer::new_for_http()))
        .nest("/yjs", yjs_routes.layer(TraceLayer::new_for_http()))
        .fallback_service(ServeDir::new(&static_dir).fallback(ServeFile::new(format!("{}/index.html", static_dir))))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    tracing::info!("Server listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}


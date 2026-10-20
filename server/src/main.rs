use axum_extra::extract::cookie::Key;
use sqlx::AnyPool;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{broadcast, Mutex};
use yrs_axum::broadcast::BroadcastGroup;

use desktop::devices::{DeviceEvent, DevicePresence};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod accounts;
mod admin;
mod api;
mod cluster;
mod compile;
mod db;
mod desktop;
mod drive;
mod email;
mod file_storage;
mod hashing;
mod models;
mod packages;
mod realtime;
mod request_context;
mod routes;
mod sharing;
mod site_url;
mod teams;

use compile::compiler::TypstCompiler;

pub type RateLimiterMap = Arc<Mutex<HashMap<String, (u32, std::time::Instant)>>>;

#[derive(Clone)]
pub struct AppState {
    pub compiler: Arc<Mutex<TypstCompiler>>,
    pub bcast_map: Arc<Mutex<HashMap<String, Arc<BroadcastGroup>>>>,
    pub db: AnyPool,
    pub key: Key,
    pub rate_limiter: RateLimiterMap,
    pub device_presence: Arc<Mutex<HashMap<String, DevicePresence>>>,
    pub device_events: Arc<Mutex<HashMap<String, broadcast::Sender<DeviceEvent>>>>,
    pub live_sessions: Arc<Mutex<sharing::access::LiveSessions>>,
    pub live_documents: Arc<Mutex<realtime::live_documents::LiveDocuments>>,
    pub cluster: Arc<cluster::Cluster>,
    pub file_storage: Arc<file_storage::FileStorage>,
    pub login_guard: Arc<accounts::login_guard::LoginGuard>,
    pub mailer: Arc<email::Mailer>,
    pub sso: Arc<accounts::sso::SsoRegistry>,
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
    let _ = rustls::crypto::ring::default_provider().install_default();

    let db = db::init_db().await;

    let key = match std::env::var("COOKIE_SECRET") {
        Ok(secret) => {
            let bytes = secret.as_bytes();
            if bytes.len() < 64 {
                tracing::warn!("COOKIE_SECRET is shorter than 64 bytes; sessions will not persist across restarts");
                Key::generate()
            } else {
                Key::from(bytes)
            }
        }
        Err(_) => {
            tracing::warn!("COOKIE_SECRET not set; generating a random key. Sessions will be invalidated on restart.");
            Key::generate()
        }
    };

    let state = AppState {
        compiler: Arc::new(Mutex::new(TypstCompiler::new())),
        bcast_map: Arc::new(Mutex::new(HashMap::new())),
        db,
        key,
        rate_limiter: Arc::new(Mutex::new(HashMap::new())),
        device_presence: Arc::new(Mutex::new(HashMap::new())),
        device_events: Arc::new(Mutex::new(HashMap::new())),
        live_sessions: Arc::new(Mutex::new(HashMap::new())),
        live_documents: Arc::new(Mutex::new(HashMap::new())),
        cluster: Arc::new(cluster::Cluster::from_environment()),
        file_storage: Arc::new(file_storage::FileStorage::from_environment()),
        login_guard: Arc::new(accounts::login_guard::LoginGuard::default()),
        mailer: Arc::new(email::Mailer::from_environment()),
        sso: Arc::new(accounts::sso::SsoRegistry::from_environment()),
    };

    cluster::start(state.clone());
    file_storage::maintenance::spawn(state.clone());

    let app = routes::build_router(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap();
    tracing::info!("Server listening on http://{}", addr);
    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .await
        .unwrap();
}


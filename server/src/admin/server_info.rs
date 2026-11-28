use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Serialize;

use crate::admin::users::require_admin;
use crate::db::{self, sql};
use crate::AppState;

struct KnownVariable {
    name: &'static str,
    group: &'static str,
    description: &'static str,
}

const fn variable(name: &'static str, group: &'static str, description: &'static str) -> KnownVariable {
    KnownVariable { name, group, description }
}

const KNOWN_VARIABLES: &[KnownVariable] = &[
    variable("DATABASE_URL", "Database", "Connection URL for SQLite, PostgreSQL or MySQL/MariaDB."),
    variable("DB_TYPE", "Database", "Database type, when it cannot be detected from the URL."),
    variable("PORT", "Server", "Port the HTTP server listens on."),
    variable("STATIC_DIR", "Server", "Folder with the built web app."),
    variable("RUST_LOG", "Server", "Log filter."),
    variable("UPLOAD_MAX_MB", "Server", "The largest file people can upload, in MB. Default 25."),
    variable("REMOTE_CACHE_MB", "Server", "Memory for files that documents load from web addresses, kept for 10 minutes. Default 256."),
    variable("REMOTE_URLS_ALLOW_PRIVATE", "Server", "Let documents load images and files from private network addresses. Off by default."),
    variable("GIT_CACHE_DIR", "Server", "Folder for the disposable partial clones used to sync with Git servers other than GitHub and GitLab."),
    variable("TYPST_CACHE_MAX_IDLE_MINUTES", "Typst", "How long Typst keeps what it learnt from compiling a document nobody edits. Default 60."),
    variable("TYPST_CACHE_MAX_MEMORY_MB", "Typst", "Above this memory use, Typst keeps only what was used in the last minute. Default 2560, or 70 % of the container memory limit when that is lower."),
    variable("PREVIEW_CACHE_DIR", "Typst", "Folder for the last preview of each document and project, served at once after a restart."),
    variable("PREVIEW_CACHE_MB", "Typst", "Disk space for kept previews. Default 2000; 0 turns them off."),
    variable("TYPST_WARMUP_COUNT", "Typst", "How many recently edited projects and documents to compile after a start. Default 4; 0 turns this off."),
    variable("TYPST_WARMUP_MAX_AGE_DAYS", "Typst", "Only warm up those edited in the last days. Default 7."),
    variable("TYPST_WARMUP_PARALLEL", "Typst", "Warm-up compiles at the same time. Default 1."),
    variable("TYPST_WARMUP_DELAY_SECONDS", "Typst", "Wait after a start before the warm-up. Default 10."),
    variable("TYPST_MAX_COMPILES", "Typst", "Typst compiles that run at the same time. Default: the number of CPU cores."),
    variable("TYPST_MAX_COMPILES_PER_USER", "Typst", "Typst compiles one account or address can run at the same time. Default 3."),
    variable("TYPST_COMPILE_TIMEOUT_SECONDS", "Typst", "How long a Typst compile may take before the request gets an error. Default 120."),
    variable("LSP_MAX_SESSIONS", "Typst", "How many tinymist processes the server runs at once. Default 64."),
    variable("LSP_MAX_SESSIONS_PER_USER", "Typst", "How many tinymist processes one account can use at once. Default 4."),
    variable("TECTONIC_PATH", "LaTeX", "Path to the tectonic program. Found on the PATH when not set."),
    variable("LATEX_DIR", "LaTeX", "Folder for the LaTeX sandbox and its package cache."),
    variable("LATEX_TIMEOUT_SECONDS", "LaTeX", "How long one LaTeX compile may run before it is stopped. Default 90."),
    variable("LATEX_MAX_PARALLEL", "LaTeX", "How many LaTeX compiles may run at the same time. Default 2."),
    variable("LATEX_MEMORY_MB", "LaTeX", "Memory limit for one LaTeX compile. Default 2048."),
    variable("LATEX_BUNDLE", "LaTeX", "URL or path of a tectonic bundle to use instead of the default one."),
    variable("LATEX_SANDBOX", "LaTeX", "Set to off to run LaTeX without the sandbox. Only for servers where every user is trusted."),
    variable("COOKIE_SECRET", "Security", "Secret used to sign session cookies."),
    variable("PUBLIC_URL", "Security", "Address people use to open TypstDrive. Needed for sign-on and email links."),
    variable("TRUST_PROXY_HEADERS", "Security", "Read client addresses from X-Forwarded-For behind a reverse proxy."),
    variable("OIDC_PROVIDERS", "Single sign-on", "Comma-separated OIDC provider names, each configured with OIDC_<NAME>_ variables."),
    variable("GOOGLE_CLIENT_ID", "Single sign-on", "OAuth client for Sign in with Google."),
    variable("GOOGLE_CLIENT_SECRET", "Single sign-on", "OAuth client secret for Google."),
    variable("GOOGLE_HOSTED_DOMAIN", "Single sign-on", "Only allow Google Workspace accounts from this domain."),
    variable("GITHUB_CLIENT_ID", "Single sign-on", "OAuth app for Sign in with GitHub."),
    variable("GITHUB_CLIENT_SECRET", "Single sign-on", "OAuth app secret for GitHub."),
    variable("SMTP_PROVIDER", "Email", "Preset mail server: google, mailgun, sendgrid, ses, resend, mailjet, microsoft or fastmail. Turns on email."),
    variable("SMTP_REGION", "Email", "Region for Mailgun (us or eu) or Amazon SES (such as eu-west-1)."),
    variable("SMTP_HOST", "Email", "Mail server host name. Turns on email, or overrides the provider's server."),
    variable("SMTP_PORT", "Email", "Mail server port."),
    variable("SMTP_SECURITY", "Email", "starttls, tls or none."),
    variable("SMTP_USERNAME", "Email", "User name for the mail server. Filled in for SendGrid and Resend."),
    variable("SMTP_PASSWORD", "Email", "Password, app password or API key for the mail server."),
    variable("SMTP_FROM", "Email", "Sender address, for example TypstDrive <noreply@example.com>. Required when the user name is not an email address."),
    variable("REDIS_URL", "Real-time", "Redis or Valkey server shared by every app instance."),
    variable("REDIS_CHANNEL_PREFIX", "Real-time", "Prefix for the pub/sub channels."),
    variable("S3_BUCKET", "File storage", "Bucket for uploaded files. Turns on object storage."),
    variable("S3_REGION", "File storage", "Bucket region."),
    variable("S3_ENDPOINT", "File storage", "Endpoint for S3-compatible services such as RustFS or R2."),
    variable("S3_ACCESS_KEY_ID", "File storage", "Access key for the bucket."),
    variable("S3_SECRET_ACCESS_KEY", "File storage", "Secret key for the bucket."),
    variable("S3_PREFIX", "File storage", "Folder inside the bucket."),
    variable("S3_VIRTUAL_HOSTED_STYLE", "File storage", "Use bucket.host style URLs instead of host/bucket."),
    variable("OBJECT_CACHE_MB", "File storage", "Memory used to cache files read from object storage."),
];

#[derive(Serialize)]
pub struct EnvironmentVariableView {
    name: &'static str,
    group: &'static str,
    description: &'static str,
    is_set: bool,
}

#[derive(Serialize)]
pub struct DatabaseView {
    kind: &'static str,
    version: Option<String>,
    schema_version: i64,
    max_connections: u32,
}

#[derive(Serialize)]
pub struct RealtimeView {
    redis_enabled: bool,
    connected: bool,
    instance_id: String,
}

#[derive(Serialize)]
pub struct FileStorageView {
    object_storage_enabled: bool,
    files_in_database: i64,
    files_in_object_storage: i64,
    cache_limit_bytes: i64,
}

const TYPST_VERSION: &str = "0.15.1";

#[derive(Serialize)]
pub struct ServerInfo {
    server_version: &'static str,
    typst_version: &'static str,
    latex: LatexView,
    database: DatabaseView,
    realtime: RealtimeView,
    file_storage: FileStorageView,
    environment: Vec<EnvironmentVariableView>,
}

#[derive(Serialize)]
struct LatexView {
    engine: Option<String>,
    sandbox: Option<&'static str>,
    problem: Option<String>,
}

async fn latex_view() -> LatexView {
    let status = crate::compile::tectonic::engine_status().await;
    LatexView { engine: status.version, sandbox: status.sandbox, problem: status.problem }
}

fn is_set(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| !value.is_empty())
}

async fn file_locations(state: &AppState) -> (i64, i64) {
    sqlx::query_as::<_, (i64, i64)>(sql(
        "SELECT CAST(COALESCE(SUM(CASE WHEN storage_key IS NULL THEN 1 ELSE 0 END), 0) AS BIGINT), \
         CAST(COALESCE(SUM(CASE WHEN storage_key IS NULL THEN 0 ELSE 1 END), 0) AS BIGINT) FROM files",
    ))
    .fetch_one(&state.db)
    .await
    .unwrap_or((0, 0))
}

pub async fn server_info(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<ServerInfo>, (StatusCode, String)> {
    require_admin(&state, &jar).await?;

    let dialect = db::dialect();
    let (files_in_database, files_in_object_storage) = file_locations(&state).await;

    Ok(Json(ServerInfo {
        server_version: env!("CARGO_PKG_VERSION"),
        typst_version: TYPST_VERSION,
        latex: latex_view().await,
        database: DatabaseView {
            kind: dialect.label(),
            version: db::server_version(&state.db).await,
            schema_version: db::CURRENT_VERSION,
            max_connections: dialect.max_connections(),
        },
        realtime: RealtimeView {
            redis_enabled: state.cluster.is_enabled(),
            connected: state.cluster.is_connected(),
            instance_id: state.cluster.instance_id().chars().take(8).collect(),
        },
        file_storage: FileStorageView {
            object_storage_enabled: state.file_storage.uses_object_storage(),
            files_in_database,
            files_in_object_storage,
            cache_limit_bytes: state.file_storage.cache_limit_bytes() as i64,
        },
        environment: KNOWN_VARIABLES
            .iter()
            .map(|known| EnvironmentVariableView {
                name: known.name,
                group: known.group,
                description: known.description,
                is_set: is_set(known.name),
            })
            .collect(),
    }))
}

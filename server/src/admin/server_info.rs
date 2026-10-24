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
    variable("REMOTE_URLS_ALLOW_PRIVATE", "Server", "Let documents load images and files from private network addresses. Off by default."),
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
    latex_package: &'static str,
    database: DatabaseView,
    realtime: RealtimeView,
    file_storage: FileStorageView,
    environment: Vec<EnvironmentVariableView>,
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
        latex_package: crate::compile::latex::MITEX_PACKAGE,
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

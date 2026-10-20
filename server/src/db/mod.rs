mod dialect;
mod migrate;
mod mysql;
mod schema;

use std::sync::OnceLock;

use sqlx::any::AnyPoolOptions;
use sqlx::{AnyPool, AssertSqlSafe};

pub use dialect::Dialect;
pub use migrate::CURRENT_VERSION;

static ACTIVE_DIALECT: OnceLock<Dialect> = OnceLock::new();

pub fn dialect() -> Dialect {
    ACTIVE_DIALECT.get().copied().unwrap_or(Dialect::Postgres)
}

pub fn sql(query: &'static str) -> &'static str {
    match dialect() {
        Dialect::Mysql => mysql::translated(query),
        Dialect::Sqlite | Dialect::Postgres => query,
    }
}

pub fn sql_owned(query: String) -> AssertSqlSafe<String> {
    match dialect() {
        Dialect::Mysql => AssertSqlSafe(mysql::translate(&query)),
        Dialect::Sqlite | Dialect::Postgres => AssertSqlSafe(query),
    }
}

pub fn timestamp_now() -> String {
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub async fn server_version(pool: &AnyPool) -> Option<String> {
    sqlx::query_as::<_, (String,)>(dialect().version_query())
        .fetch_one(pool)
        .await
        .ok()
        .map(|(version,)| version)
}

pub async fn init_db() -> AnyPool {
    sqlx::any::install_default_drivers();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/typstdrive".to_string());
    let dialect = Dialect::from_environment(&database_url);
    let _ = ACTIVE_DIALECT.set(dialect);

    let pool = AnyPoolOptions::new()
        .max_connections(dialect.max_connections())
        .connect(&database_url)
        .await
        .expect("Failed to connect to database. Check DATABASE_URL.");

    migrate::migrate(&pool, dialect)
        .await
        .expect("Failed to prepare the database schema");

    pool
}

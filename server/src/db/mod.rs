mod dialect;
mod migrate;
mod schema;

use sqlx::any::AnyPoolOptions;
use sqlx::AnyPool;

use dialect::Dialect;


pub fn timestamp_now() -> String {
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub async fn init_db() -> AnyPool {
    sqlx::any::install_default_drivers();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/typstdrive".to_string());
    let dialect = Dialect::from_environment(&database_url);

    let max_connections = match dialect {
        Dialect::Sqlite => 4,
        Dialect::Postgres => 10,
    };

    let pool = AnyPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database. Check DATABASE_URL.");

    migrate::migrate(&pool, dialect)
        .await
        .expect("Failed to prepare the database schema");

    pool
}

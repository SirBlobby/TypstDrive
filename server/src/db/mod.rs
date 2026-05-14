mod postgres;
mod sqlite;

use sqlx::any::AnyPoolOptions;
use sqlx::AnyPool;

pub async fn init_db() -> AnyPool {
    sqlx::any::install_default_drivers();

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/typstdrive".to_string());

    // DB_TYPE can override URL-based detection: "sqlite" or "postgres"
    let db_type = std::env::var("DB_TYPE")
        .unwrap_or_else(|_| {
            if db_url.starts_with("sqlite") {
                "sqlite".to_string()
            } else {
                "postgres".to_string()
            }
        });

    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database. Check DATABASE_URL.");

    match db_type.as_str() {
        "sqlite" => sqlite::init_schema(&pool).await,
        "postgres" => postgres::init_schema(&pool).await,
        other => panic!("Unknown DB_TYPE '{}'. Expected 'sqlite' or 'postgres'.", other),
    }

    pool
}

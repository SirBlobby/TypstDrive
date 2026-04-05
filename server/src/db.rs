use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};

pub async fn init_db() -> Pool<Sqlite> {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:typstdrive.db?mode=rwc".to_string());
    
    // Ensure parent directory exists if there is one
    if let Some(path) = db_url.strip_prefix("sqlite:") {
        if let Some(path) = path.split('?').next() {
            if let Some(parent) = std::path::Path::new(path).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to create pool.");

    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    parent_id TEXT,
    name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(owner_id) REFERENCES users(id),
    FOREIGN KEY(parent_id) REFERENCES folders(id)
);
CREATE TABLE IF NOT EXISTS documents (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    folder_id TEXT,
    title TEXT NOT NULL,
    content BLOB,
    thumbnail_svg TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(owner_id) REFERENCES users(id),
    FOREIGN KEY(folder_id) REFERENCES folders(id)
);
CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    document_id TEXT,
    folder_id TEXT,
    name TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    data BLOB NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(owner_id) REFERENCES users(id),
    FOREIGN KEY(document_id) REFERENCES documents(id),
    FOREIGN KEY(folder_id) REFERENCES folders(id)
);
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to initialize database schema");

    
    let _ = sqlx::query("ALTER TABLE documents ADD COLUMN folder_id TEXT REFERENCES folders(id)")
        .execute(&pool)
        .await;

    
    let _ = sqlx::query("ALTER TABLE documents ADD COLUMN thumbnail_svg TEXT")
        .execute(&pool)
        .await;

    let _ = sqlx::query("ALTER TABLE files ADD COLUMN folder_id TEXT REFERENCES folders(id)")
        .execute(&pool)
        .await;

    pool
}

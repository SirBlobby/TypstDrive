use sqlx::AnyPool;

pub async fn init_schema(pool: &AnyPool) {
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await
        .expect("Failed to enable SQLite foreign keys");

    let statements = [
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            email TEXT UNIQUE,
            password_hash TEXT NOT NULL
        )",
        "CREATE TABLE IF NOT EXISTS folders (
            id TEXT PRIMARY KEY,
            owner_id TEXT NOT NULL REFERENCES users(id),
            parent_id TEXT REFERENCES folders(id),
            name TEXT NOT NULL,
            created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now'))
        )",
        "CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            owner_id TEXT NOT NULL REFERENCES users(id),
            folder_id TEXT REFERENCES folders(id),
            title TEXT NOT NULL,
            content BLOB,
            thumbnail_svg TEXT,
            public_role TEXT DEFAULT NULL,
            created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now')),
            updated_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now'))
        )",
        "CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            owner_id TEXT NOT NULL REFERENCES users(id),
            document_id TEXT REFERENCES documents(id),
            folder_id TEXT REFERENCES folders(id),
            name TEXT NOT NULL,
            mime_type TEXT NOT NULL,
            data BLOB NOT NULL,
            created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now'))
        )",
        "CREATE TABLE IF NOT EXISTS collaborators (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            role TEXT NOT NULL,
            created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now')),
            UNIQUE(document_id, user_id)
        )",
        "CREATE TABLE IF NOT EXISTS invitations (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            role TEXT NOT NULL,
            token TEXT NOT NULL UNIQUE,
            created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now')),
            expires_at TEXT
        )",
        "CREATE TABLE IF NOT EXISTS comments (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            resolved INTEGER DEFAULT 0,
            created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now'))
        )",
        "CREATE TABLE IF NOT EXISTS document_history (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            content BLOB NOT NULL,
            created_by TEXT NOT NULL REFERENCES users(id) ON DELETE SET NULL,
            created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now'))
        )",
        "CREATE TABLE IF NOT EXISTS document_versions (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now'))
        )",
    ];

    for stmt in &statements {
        sqlx::query(stmt)
            .execute(pool)
            .await
            .expect("Failed to execute SQLite schema");
    }
}

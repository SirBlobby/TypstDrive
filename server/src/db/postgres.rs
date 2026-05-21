use sqlx::AnyPool;

pub async fn init_schema(pool: &AnyPool) {
    let statements = [
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            email TEXT UNIQUE,
            password_hash TEXT NOT NULL,
            is_admin BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TEXT DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS')
        )",
        "CREATE TABLE IF NOT EXISTS folders (
            id TEXT PRIMARY KEY,
            owner_id TEXT NOT NULL REFERENCES users(id),
            parent_id TEXT REFERENCES folders(id),
            name TEXT NOT NULL,
            created_at TEXT DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS')
        )",
        "CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            owner_id TEXT NOT NULL REFERENCES users(id),
            folder_id TEXT REFERENCES folders(id),
            title TEXT NOT NULL,
            content BYTEA,
            thumbnail_svg TEXT,
            public_role TEXT DEFAULT NULL,
            created_at TEXT DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS'),
            updated_at TEXT DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS')
        )",
        "CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            owner_id TEXT NOT NULL REFERENCES users(id),
            document_id TEXT REFERENCES documents(id),
            folder_id TEXT REFERENCES folders(id),
            name TEXT NOT NULL,
            mime_type TEXT NOT NULL,
            data BYTEA NOT NULL,
            created_at TEXT DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS')
        )",
        "CREATE TABLE IF NOT EXISTS collaborators (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            role TEXT NOT NULL,
            created_at TEXT DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS'),
            UNIQUE(document_id, user_id)
        )",
        "CREATE TABLE IF NOT EXISTS invitations (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            role TEXT NOT NULL,
            token TEXT NOT NULL UNIQUE,
            created_at TEXT DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS'),
            expires_at TEXT
        )",
        "CREATE TABLE IF NOT EXISTS comments (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            resolved BOOLEAN DEFAULT FALSE,
            created_at TEXT DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS')
        )",
        "CREATE TABLE IF NOT EXISTS document_history (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            content BYTEA NOT NULL,
            created_by TEXT NOT NULL REFERENCES users(id) ON DELETE SET NULL,
            created_at TEXT DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS')
        )",
        "CREATE TABLE IF NOT EXISTS document_versions (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            created_at TEXT DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS')
        )",
        "CREATE TABLE IF NOT EXISTS api_keys (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            key_hash TEXT NOT NULL UNIQUE,
            key_prefix TEXT NOT NULL,
            created_at TEXT NOT NULL,
            last_used_at TEXT,
            rate_limit INTEGER NOT NULL DEFAULT 60
        )",
        "CREATE TABLE IF NOT EXISTS api_render_cache (
            id TEXT PRIMARY KEY,
            content_hash TEXT NOT NULL UNIQUE,
            format TEXT NOT NULL,
            data BYTEA NOT NULL,
            created_at TEXT NOT NULL
        )",
        "CREATE TABLE IF NOT EXISTS api_key_usage (
            key_id TEXT NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
            date TEXT NOT NULL,
            count INTEGER NOT NULL DEFAULT 1,
            PRIMARY KEY(key_id, date)
        )",
        "CREATE TABLE IF NOT EXISTS api_key_usage_detail (
            key_id TEXT NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
            minute TEXT NOT NULL,
            count INTEGER NOT NULL DEFAULT 1,
            PRIMARY KEY(key_id, minute)
        )",
    ];

    for stmt in &statements {
        sqlx::query(stmt)
            .execute(pool)
            .await
            .expect("Failed to execute Postgres schema");
    }

    // Idempotent migrations for existing databases
    let migrations = [
        "ALTER TABLE documents ADD COLUMN IF NOT EXISTS public_role TEXT",
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS is_admin BOOLEAN NOT NULL DEFAULT FALSE",
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS created_at TEXT DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS')",
    ];
    for stmt in &migrations {
        sqlx::query(stmt).execute(pool).await.unwrap_or_else(|_| Default::default());
    }
}

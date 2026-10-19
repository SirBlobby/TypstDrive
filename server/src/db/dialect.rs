#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    Sqlite,
    Postgres,
}

impl Dialect {
    pub fn from_environment(database_url: &str) -> Self {
        let configured = std::env::var("DB_TYPE").ok();
        match configured.as_deref() {
            Some("sqlite") => Self::Sqlite,
            Some("postgres") => Self::Postgres,
            Some(other) => panic!("Unknown DB_TYPE '{other}'. Expected 'sqlite' or 'postgres'."),
            None if database_url.starts_with("sqlite") => Self::Sqlite,
            None => Self::Postgres,
        }
    }

    fn blob_type(self) -> &'static str {
        match self {
            Self::Sqlite => "BLOB",
            Self::Postgres => "BYTEA",
        }
    }

    fn integer_type(self) -> &'static str {
        match self {
            Self::Sqlite => "INTEGER",
            Self::Postgres => "BIGINT",
        }
    }

    pub fn now_expression(self) -> &'static str {
        match self {
            Self::Sqlite => "(strftime('%Y-%m-%d %H:%M:%S', 'now'))",
            Self::Postgres => "(to_char(timezone('UTC', now()), 'YYYY-MM-DD HH24:MI:SS'))",
        }
    }

    pub fn drop_table(self, table: &str) -> String {
        match self {
            Self::Sqlite => format!("DROP TABLE IF EXISTS {table}"),
            Self::Postgres => format!("DROP TABLE IF EXISTS {table} CASCADE"),
        }
    }

    pub fn table_exists_query(self) -> &'static str {
        match self {
            Self::Sqlite => "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = $1",
            Self::Postgres => "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = current_schema() AND table_name = $1",
        }
    }

    pub fn column_names_query(self) -> &'static str {
        match self {
            Self::Sqlite => "SELECT name FROM pragma_table_info($1)",
            Self::Postgres => "SELECT CAST(column_name AS TEXT) FROM information_schema.columns WHERE table_schema = current_schema() AND table_name = $1",
        }
    }

    pub fn render(self, definition: &str, own_table: &str) -> String {
        definition
            .replace("{blob}", self.blob_type())
            .replace("{integer}", self.integer_type())
            .replace("{now}", self.now_expression())
            .replace("{self}", own_table)
    }
}

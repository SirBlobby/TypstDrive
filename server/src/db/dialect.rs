#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    Sqlite,
    Postgres,
    Mysql,
}

impl Dialect {
    pub fn from_environment(database_url: &str) -> Self {
        let configured = std::env::var("DB_TYPE").ok();
        match configured.as_deref() {
            Some("sqlite") => Self::Sqlite,
            Some("postgres") => Self::Postgres,
            Some("mysql") | Some("mariadb") => Self::Mysql,
            Some(other) => panic!("Unknown DB_TYPE '{other}'. Expected 'sqlite', 'postgres', 'mysql' or 'mariadb'."),
            None if database_url.starts_with("sqlite") => Self::Sqlite,
            None if database_url.starts_with("mysql") || database_url.starts_with("mariadb") => Self::Mysql,
            None => Self::Postgres,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Sqlite => "SQLite",
            Self::Postgres => "PostgreSQL",
            Self::Mysql => "MySQL / MariaDB",
        }
    }

    pub fn max_connections(self) -> u32 {
        match self {
            Self::Sqlite => 4,
            Self::Postgres | Self::Mysql => 10,
        }
    }

    fn blob_type(self) -> &'static str {
        match self {
            Self::Sqlite => "BLOB",
            Self::Postgres => "BYTEA",
            Self::Mysql => "LONGBLOB",
        }
    }

    fn integer_type(self) -> &'static str {
        match self {
            Self::Sqlite => "INTEGER",
            Self::Postgres | Self::Mysql => "BIGINT",
        }
    }

    pub fn now_expression(self) -> &'static str {
        match self {
            Self::Sqlite => "(strftime('%Y-%m-%d %H:%M:%S', 'now'))",
            Self::Postgres => "(to_char(timezone('UTC', now()), 'YYYY-MM-DD HH24:MI:SS'))",
            Self::Mysql => "(DATE_FORMAT(UTC_TIMESTAMP(), '%Y-%m-%d %H:%M:%S'))",
        }
    }

    pub fn drop_table(self, table: &str) -> String {
        match self {
            Self::Postgres => format!("DROP TABLE IF EXISTS {table} CASCADE"),
            Self::Sqlite | Self::Mysql => format!("DROP TABLE IF EXISTS {table}"),
        }
    }

    pub fn table_exists_query(self) -> &'static str {
        match self {
            Self::Sqlite => "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = $1",
            Self::Postgres => "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = current_schema() AND table_name = $1",
            Self::Mysql => "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = DATABASE() AND table_name = ?",
        }
    }

    pub fn column_names_query(self) -> &'static str {
        match self {
            Self::Sqlite => "SELECT name FROM pragma_table_info($1)",
            Self::Postgres => "SELECT CAST(column_name AS TEXT) FROM information_schema.columns WHERE table_schema = current_schema() AND table_name = $1",
            Self::Mysql => "SELECT CAST(column_name AS CHAR) FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = ?",
        }
    }

    pub fn index_exists_query(self) -> Option<&'static str> {
        match self {
            Self::Mysql => Some("SELECT COUNT(*) FROM information_schema.statistics WHERE table_schema = DATABASE() AND index_name = ?"),
            Self::Sqlite | Self::Postgres => None,
        }
    }

    pub fn version_query(self) -> &'static str {
        match self {
            Self::Sqlite => "SELECT sqlite_version()",
            Self::Postgres => "SELECT CAST(version() AS TEXT)",
            Self::Mysql => "SELECT CAST(VERSION() AS CHAR)",
        }
    }

    pub fn table_options(self) -> &'static str {
        match self {
            Self::Mysql => " ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin",
            Self::Sqlite | Self::Postgres => "",
        }
    }

    pub fn render(self, definition: &str, own_table: &str) -> String {
        definition
            .replace("{blob}", self.blob_type())
            .replace("{integer}", self.integer_type())
            .replace("{now}", self.now_expression())
            .replace("{self}", own_table)
    }

    pub fn render_column(self, definition: &str, own_table: &str, is_key: bool) -> String {
        let rendered = self.render(definition, own_table);
        if self != Self::Mysql {
            return rendered;
        }
        let Some(rest) = rendered.strip_prefix("TEXT") else {
            return rendered;
        };
        let needs_length = is_key || ["PRIMARY KEY", "UNIQUE", "REFERENCES", "DEFAULT"].iter().any(|word| rest.contains(word));
        if needs_length {
            format!("VARCHAR(255){rest}")
        } else {
            format!("LONGTEXT{rest}")
        }
    }
}

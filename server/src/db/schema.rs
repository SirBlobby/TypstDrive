pub struct Column {
    pub name: &'static str,
    pub definition: &'static str,
    pub copy: CopyRule,
}

#[derive(Clone, Copy)]
pub enum CopyRule {
    Plain,
    DefaultWhenNull(&'static str),
    Boolean,
    OptionalReference(&'static str),
    RequiredReference(&'static str),
}

pub struct Table {
    pub name: &'static str,
    pub columns: &'static [Column],
    pub constraints: &'static [&'static str],
}

const fn plain(name: &'static str, definition: &'static str) -> Column {
    Column { name, definition, copy: CopyRule::Plain }
}

const fn created_at(name: &'static str) -> Column {
    Column { name, definition: "TEXT NOT NULL DEFAULT {now}", copy: CopyRule::DefaultWhenNull("{now}") }
}

const fn flag(name: &'static str) -> Column {
    Column { name, definition: "{integer} NOT NULL DEFAULT 0", copy: CopyRule::Boolean }
}

const fn required_reference(name: &'static str, parent: &'static str, definition: &'static str) -> Column {
    Column { name, definition, copy: CopyRule::RequiredReference(parent) }
}

const fn optional_reference(name: &'static str, parent: &'static str, definition: &'static str) -> Column {
    Column { name, definition, copy: CopyRule::OptionalReference(parent) }
}

pub const TABLES: &[Table] = &[
    Table {
        name: "users",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            plain("username", "TEXT NOT NULL UNIQUE"),
            plain("email", "TEXT UNIQUE"),
            plain("password_hash", "TEXT NOT NULL"),
            flag("is_admin"),
            created_at("created_at"),
        ],
        constraints: &[],
    },
    Table {
        name: "site_settings",
        columns: &[
            plain("setting_key", "TEXT PRIMARY KEY"),
            plain("setting_value", "TEXT NOT NULL"),
        ],
        constraints: &[],
    },
    Table {
        name: "user_api_limits",
        columns: &[
            Column { name: "user_id", definition: "TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE", copy: CopyRule::RequiredReference("users") },
            plain("api_enabled", "{integer}"),
            plain("rate_limit", "{integer}"),
            plain("max_keys", "{integer}"),
            plain("daily_quota", "{integer}"),
            created_at("updated_at"),
        ],
        constraints: &[],
    },
    Table {
        name: "user_storage_quotas",
        columns: &[
            Column { name: "user_id", definition: "TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE", copy: CopyRule::RequiredReference("users") },
            plain("quota_bytes", "{integer} NOT NULL"),
            created_at("updated_at"),
        ],
        constraints: &[],
    },
    Table {
        name: "folders",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("owner_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            optional_reference("parent_id", "folders", "TEXT REFERENCES {self}(id) ON DELETE CASCADE"),
            plain("name", "TEXT NOT NULL"),
            created_at("created_at"),
        ],
        constraints: &[],
    },
    Table {
        name: "documents",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("owner_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            optional_reference("folder_id", "folders", "TEXT REFERENCES folders(id) ON DELETE CASCADE"),
            plain("title", "TEXT NOT NULL"),
            plain("content", "{blob}"),
            plain("thumbnail_svg", "TEXT"),
            plain("public_role", "TEXT"),
            created_at("created_at"),
            created_at("updated_at"),
        ],
        constraints: &[],
    },
    Table {
        name: "files",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("owner_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            optional_reference("document_id", "documents", "TEXT REFERENCES documents(id) ON DELETE SET NULL"),
            optional_reference("folder_id", "folders", "TEXT REFERENCES folders(id) ON DELETE CASCADE"),
            plain("name", "TEXT NOT NULL"),
            plain("mime_type", "TEXT NOT NULL"),
            plain("data", "{blob} NOT NULL"),
            Column { name: "size_bytes", definition: "{integer} NOT NULL DEFAULT 0", copy: CopyRule::DefaultWhenNull("0") },
            plain("storage_key", "TEXT"),
            created_at("created_at"),
        ],
        constraints: &[],
    },
    Table {
        name: "collaborators",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("document_id", "documents", "TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE"),
            required_reference("user_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            plain("role", "TEXT NOT NULL"),
            created_at("created_at"),
        ],
        constraints: &["UNIQUE (document_id, user_id)"],
    },
    Table {
        name: "comments",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("document_id", "documents", "TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE"),
            required_reference("user_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            plain("content", "TEXT NOT NULL"),
            flag("resolved"),
            created_at("created_at"),
        ],
        constraints: &[],
    },
    Table {
        name: "document_versions",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("document_id", "documents", "TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE"),
            required_reference("user_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            plain("content", "TEXT NOT NULL"),
            created_at("created_at"),
        ],
        constraints: &[],
    },
    Table {
        name: "api_keys",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("user_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            plain("name", "TEXT NOT NULL"),
            plain("key_hash", "TEXT NOT NULL UNIQUE"),
            plain("key_prefix", "TEXT NOT NULL"),
            created_at("created_at"),
            plain("last_used_at", "TEXT"),
            Column { name: "rate_limit", definition: "{integer} NOT NULL DEFAULT 60", copy: CopyRule::DefaultWhenNull("60") },
        ],
        constraints: &[],
    },
    Table {
        name: "api_render_cache",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            plain("content_hash", "TEXT NOT NULL UNIQUE"),
            plain("format", "TEXT NOT NULL"),
            plain("data", "{blob} NOT NULL"),
            created_at("created_at"),
        ],
        constraints: &[],
    },
    Table {
        name: "api_key_usage",
        columns: &[
            required_reference("key_id", "api_keys", "TEXT NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE"),
            plain("date", "TEXT NOT NULL"),
            Column { name: "count", definition: "{integer} NOT NULL DEFAULT 1", copy: CopyRule::DefaultWhenNull("1") },
        ],
        constraints: &["PRIMARY KEY (key_id, date)"],
    },
    Table {
        name: "api_key_usage_detail",
        columns: &[
            required_reference("key_id", "api_keys", "TEXT NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE"),
            plain("minute", "TEXT NOT NULL"),
            Column { name: "count", definition: "{integer} NOT NULL DEFAULT 1", copy: CopyRule::DefaultWhenNull("1") },
        ],
        constraints: &["PRIMARY KEY (key_id, minute)"],
    },
    Table {
        name: "projects",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("owner_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            optional_reference("folder_id", "folders", "TEXT REFERENCES folders(id) ON DELETE CASCADE"),
            plain("name", "TEXT NOT NULL"),
            Column { name: "entrypoint", definition: "TEXT NOT NULL DEFAULT 'main.typ'", copy: CopyRule::DefaultWhenNull("'main.typ'") },
            plain("thumbnail_svg", "TEXT"),
            plain("public_role", "TEXT"),
            created_at("created_at"),
            created_at("updated_at"),
        ],
        constraints: &[],
    },
    Table {
        name: "project_files",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("project_id", "projects", "TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE"),
            plain("path", "TEXT NOT NULL"),
            Column { name: "kind", definition: "TEXT NOT NULL DEFAULT 'text'", copy: CopyRule::DefaultWhenNull("'text'") },
            plain("content", "{blob}"),
            Column { name: "mime_type", definition: "TEXT NOT NULL DEFAULT 'text/plain'", copy: CopyRule::DefaultWhenNull("'text/plain'") },
            created_at("created_at"),
            created_at("updated_at"),
        ],
        constraints: &["UNIQUE (project_id, path)"],
    },
    Table {
        name: "project_collaborators",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("project_id", "projects", "TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE"),
            required_reference("user_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            plain("role", "TEXT NOT NULL"),
            created_at("created_at"),
        ],
        constraints: &["UNIQUE (project_id, user_id)"],
    },
    Table {
        name: "folder_collaborators",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("folder_id", "folders", "TEXT NOT NULL REFERENCES folders(id) ON DELETE CASCADE"),
            required_reference("user_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            plain("role", "TEXT NOT NULL"),
            created_at("created_at"),
        ],
        constraints: &["UNIQUE (folder_id, user_id)"],
    },
    Table {
        name: "document_shared_files",
        columns: &[
            required_reference("document_id", "documents", "TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE"),
            required_reference("file_id", "files", "TEXT NOT NULL REFERENCES files(id) ON DELETE CASCADE"),
            created_at("created_at"),
        ],
        constraints: &["PRIMARY KEY (document_id, file_id)"],
    },
    Table {
        name: "ownership_transfers",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            plain("item_kind", "TEXT NOT NULL"),
            plain("item_id", "TEXT NOT NULL"),
            required_reference("from_user_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            required_reference("to_user_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            created_at("created_at"),
        ],
        constraints: &["UNIQUE (item_kind, item_id)"],
    },
    Table {
        name: "packages",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("owner_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            Column { name: "namespace", definition: "TEXT NOT NULL DEFAULT 'typstdrive'", copy: CopyRule::DefaultWhenNull("'typstdrive'") },
            plain("name", "TEXT NOT NULL"),
            plain("description", "TEXT"),
            created_at("created_at"),
        ],
        constraints: &["UNIQUE (namespace, name)"],
    },
    Table {
        name: "package_versions",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("package_id", "packages", "TEXT NOT NULL REFERENCES packages(id) ON DELETE CASCADE"),
            plain("version", "TEXT NOT NULL"),
            Column { name: "entrypoint", definition: "TEXT NOT NULL DEFAULT 'lib.typ'", copy: CopyRule::DefaultWhenNull("'lib.typ'") },
            plain("manifest", "{blob}"),
            created_at("created_at"),
        ],
        constraints: &["UNIQUE (package_id, version)"],
    },
    Table {
        name: "package_files",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("version_id", "package_versions", "TEXT NOT NULL REFERENCES package_versions(id) ON DELETE CASCADE"),
            plain("path", "TEXT NOT NULL"),
            plain("data", "{blob} NOT NULL"),
        ],
        constraints: &["UNIQUE (version_id, path)"],
    },
    Table {
        name: "device_tokens",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("user_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            plain("name", "TEXT NOT NULL"),
            plain("token_hash", "TEXT NOT NULL UNIQUE"),
            created_at("created_at"),
            plain("last_used_at", "TEXT"),
        ],
        constraints: &[],
    },
    Table {
        name: "user_avatars",
        columns: &[
            Column { name: "user_id", definition: "TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE", copy: CopyRule::RequiredReference("users") },
            plain("data", "{blob} NOT NULL"),
            created_at("updated_at"),
        ],
        constraints: &[],
    },
];

pub const INDEXES: &[&str] = &[
    "CREATE INDEX IF NOT EXISTS idx_folders_owner_parent ON folders (owner_id, parent_id)",
    "CREATE INDEX IF NOT EXISTS idx_folders_parent ON folders (parent_id)",
    "CREATE INDEX IF NOT EXISTS idx_documents_owner_folder ON documents (owner_id, folder_id)",
    "CREATE INDEX IF NOT EXISTS idx_documents_folder ON documents (folder_id)",
    "CREATE INDEX IF NOT EXISTS idx_files_owner_folder ON files (owner_id, folder_id)",
    "CREATE INDEX IF NOT EXISTS idx_files_folder ON files (folder_id)",
    "CREATE INDEX IF NOT EXISTS idx_files_document ON files (document_id)",
    "CREATE INDEX IF NOT EXISTS idx_collaborators_user ON collaborators (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_comments_document ON comments (document_id)",
    "CREATE INDEX IF NOT EXISTS idx_comments_user ON comments (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_document_versions_document ON document_versions (document_id)",
    "CREATE INDEX IF NOT EXISTS idx_document_versions_user ON document_versions (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_api_keys_user ON api_keys (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_projects_owner_folder ON projects (owner_id, folder_id)",
    "CREATE INDEX IF NOT EXISTS idx_projects_folder ON projects (folder_id)",
    "CREATE INDEX IF NOT EXISTS idx_project_collaborators_user ON project_collaborators (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_folder_collaborators_user ON folder_collaborators (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_document_shared_files_file ON document_shared_files (file_id)",
    "CREATE INDEX IF NOT EXISTS idx_ownership_transfers_to_user ON ownership_transfers (to_user_id)",
    "CREATE INDEX IF NOT EXISTS idx_packages_owner ON packages (owner_id)",
    "CREATE INDEX IF NOT EXISTS idx_device_tokens_user ON device_tokens (user_id)",
];

pub const BACKFILLS: &[&str] = &["UPDATE files SET size_bytes = LENGTH(data) WHERE storage_key IS NULL"];

pub const RETIRED_TABLES: &[&str] = &["document_history", "invitations"];

pub const LEGACY_RENAMES: &[&str] = &[
    "ALTER TABLE spaces RENAME TO projects",
    "ALTER TABLE space_files RENAME TO project_files",
    "ALTER TABLE space_collaborators RENAME TO project_collaborators",
    "ALTER TABLE project_files RENAME COLUMN space_id TO project_id",
    "ALTER TABLE project_collaborators RENAME COLUMN space_id TO project_id",
];

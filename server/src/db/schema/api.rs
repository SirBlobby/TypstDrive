use super::{Column, CopyRule, Table, plain, created_at, required_reference};

pub const TABLES: &[Table] = &[
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
            Column { name: "rate_limit", definition: "{integer} NOT NULL DEFAULT 0", copy: CopyRule::DefaultWhenNull("0") },
            Column { name: "scopes", definition: "TEXT NOT NULL DEFAULT 'render'", copy: CopyRule::DefaultWhenNull("'render'") },
            plain("expires_at", "TEXT"),
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
        name: "packages",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("owner_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            Column { name: "namespace", definition: "TEXT NOT NULL DEFAULT 'typstdrive'", copy: CopyRule::DefaultWhenNull("'typstdrive'") },
            plain("name", "TEXT NOT NULL"),
            plain("description", "TEXT"),
            Column { name: "visibility", definition: "TEXT NOT NULL DEFAULT 'public'", copy: CopyRule::DefaultWhenNull("'public'") },
            created_at("created_at"),
        ],
        constraints: &["UNIQUE (namespace, name)"],
    },
    Table {
        name: "package_members",
        columns: &[
            required_reference("package_id", "packages", "TEXT NOT NULL REFERENCES packages(id) ON DELETE CASCADE"),
            required_reference("user_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            created_at("created_at"),
        ],
        constraints: &["PRIMARY KEY (package_id, user_id)"],
    },
    Table {
        name: "package_teams",
        columns: &[
            required_reference("package_id", "packages", "TEXT NOT NULL REFERENCES packages(id) ON DELETE CASCADE"),
            required_reference("team_id", "teams", "TEXT NOT NULL REFERENCES teams(id) ON DELETE CASCADE"),
            created_at("created_at"),
        ],
        constraints: &["PRIMARY KEY (package_id, team_id)"],
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
];

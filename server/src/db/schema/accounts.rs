use super::{Column, CopyRule, Table, plain, created_at, flag, optional_reference, required_reference};

pub const TABLES: &[Table] = &[
    Table {
        name: "users",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            plain("username", "TEXT NOT NULL UNIQUE"),
            plain("email", "TEXT UNIQUE"),
            plain("password_hash", "TEXT NOT NULL"),
            flag("is_admin"),
            flag("email_verified"),
            Column { name: "account_kind", definition: "TEXT NOT NULL DEFAULT 'user'", copy: CopyRule::DefaultWhenNull("'user'") },
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
        name: "site_assets",
        columns: &[
            plain("asset_name", "TEXT PRIMARY KEY"),
            plain("data", "{blob} NOT NULL"),
            created_at("updated_at"),
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
        name: "user_avatars",
        columns: &[
            Column { name: "user_id", definition: "TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE", copy: CopyRule::RequiredReference("users") },
            plain("data", "{blob} NOT NULL"),
            created_at("updated_at"),
        ],
        constraints: &[],
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
        name: "user_sessions",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("user_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            plain("token_hash", "TEXT NOT NULL UNIQUE"),
            plain("user_agent", "TEXT"),
            plain("ip_address", "TEXT"),
            created_at("created_at"),
            created_at("last_seen_at"),
            plain("expires_at", "TEXT NOT NULL"),
        ],
        constraints: &[],
    },
    Table {
        name: "email_tokens",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("user_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            plain("token_hash", "TEXT NOT NULL UNIQUE"),
            plain("purpose", "TEXT NOT NULL"),
            plain("email", "TEXT NOT NULL"),
            created_at("created_at"),
            plain("expires_at", "TEXT NOT NULL"),
        ],
        constraints: &[],
    },
    Table {
        name: "user_identities",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            required_reference("user_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            plain("provider", "TEXT NOT NULL"),
            plain("subject", "TEXT NOT NULL"),
            plain("email", "TEXT"),
            created_at("created_at"),
        ],
        constraints: &["UNIQUE (provider, subject)"],
    },
    Table {
        name: "audit_log",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            optional_reference("actor_id", "users", "TEXT REFERENCES users(id) ON DELETE SET NULL"),
            plain("actor_name", "TEXT NOT NULL"),
            plain("action", "TEXT NOT NULL"),
            plain("target", "TEXT"),
            plain("details", "TEXT"),
            plain("ip_address", "TEXT"),
            created_at("created_at"),
        ],
        constraints: &[],
    },
    Table {
        name: "teams",
        columns: &[
            Column { name: "id", definition: "TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE", copy: CopyRule::RequiredReference("users") },
            plain("name", "TEXT NOT NULL"),
            optional_reference("created_by", "users", "TEXT REFERENCES users(id) ON DELETE SET NULL"),
            created_at("created_at"),
        ],
        constraints: &[],
    },
    Table {
        name: "team_members",
        columns: &[
            required_reference("team_id", "teams", "TEXT NOT NULL REFERENCES teams(id) ON DELETE CASCADE"),
            required_reference("user_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            plain("role", "TEXT NOT NULL"),
            created_at("created_at"),
        ],
        constraints: &["PRIMARY KEY (team_id, user_id)"],
    },
];

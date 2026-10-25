use super::{Column, CopyRule, Table, plain, created_at, required_reference};

pub const TABLES: &[Table] = &[
    Table {
        name: "git_links",
        columns: &[
            required_reference("project_id", "projects", "TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE"),
            required_reference("created_by", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            plain("provider", "TEXT NOT NULL"),
            plain("repo_url", "TEXT NOT NULL"),
            plain("branch", "TEXT NOT NULL"),
            Column { name: "folder", definition: "TEXT NOT NULL DEFAULT ''", copy: CopyRule::DefaultWhenNull("''") },
            plain("username", "TEXT"),
            plain("token_sealed", "TEXT"),
            plain("excluded_paths", "TEXT"),
            plain("base_commit", "TEXT"),
            plain("last_synced_at", "TEXT"),
            created_at("created_at"),
        ],
        constraints: &[],
    },
    Table {
        name: "git_link_files",
        columns: &[
            required_reference("project_id", "projects", "TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE"),
            plain("path", "TEXT NOT NULL"),
            plain("blob_sha", "TEXT NOT NULL"),
            plain("content_hash", "TEXT NOT NULL"),
        ],
        constraints: &["PRIMARY KEY (project_id, path)"],
    },
];

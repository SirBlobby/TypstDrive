use super::{Table, plain, created_at, required_reference};

pub const TABLES: &[Table] = &[
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
        name: "document_stars",
        columns: &[
            required_reference("user_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            required_reference("document_id", "documents", "TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE"),
            created_at("created_at"),
        ],
        constraints: &["PRIMARY KEY (user_id, document_id)"],
    },
    Table {
        name: "project_stars",
        columns: &[
            required_reference("user_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            required_reference("project_id", "projects", "TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE"),
            created_at("created_at"),
        ],
        constraints: &["PRIMARY KEY (user_id, project_id)"],
    },
];

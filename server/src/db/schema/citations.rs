use super::{Column, CopyRule, Table, plain, created_at, optional_reference, required_reference};

pub const TABLES: &[Table] = &[
    Table {
        name: "zotero_accounts",
        columns: &[
            required_reference("user_id", "users", "TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE"),
            plain("zotero_user_id", "TEXT NOT NULL"),
            plain("username", "TEXT NOT NULL"),
            plain("api_key_sealed", "TEXT NOT NULL"),
            created_at("created_at"),
        ],
        constraints: &[],
    },
    Table {
        name: "citation_syncs",
        columns: &[
            plain("id", "TEXT PRIMARY KEY"),
            optional_reference("document_id", "documents", "TEXT REFERENCES documents(id) ON DELETE CASCADE"),
            optional_reference("project_id", "projects", "TEXT REFERENCES projects(id) ON DELETE CASCADE"),
            required_reference("created_by", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
            plain("library_id", "TEXT NOT NULL"),
            plain("library_name", "TEXT NOT NULL"),
            plain("collection_key", "TEXT"),
            plain("collection_name", "TEXT"),
            plain("file_name", "TEXT NOT NULL"),
            Column { name: "item_count", definition: "{integer} NOT NULL DEFAULT 0", copy: CopyRule::DefaultWhenNull("0") },
            plain("last_synced_at", "TEXT"),
            created_at("created_at"),
        ],
        constraints: &["UNIQUE (document_id, file_name)", "UNIQUE (project_id, file_name)"],
    },
];

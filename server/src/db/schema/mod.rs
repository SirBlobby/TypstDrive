mod accounts;
mod api;
mod content;
mod sharing;

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

pub(super) const fn plain(name: &'static str, definition: &'static str) -> Column {
    Column { name, definition, copy: CopyRule::Plain }
}

pub(super) const fn created_at(name: &'static str) -> Column {
    Column { name, definition: "TEXT NOT NULL DEFAULT {now}", copy: CopyRule::DefaultWhenNull("{now}") }
}

pub(super) const fn flag(name: &'static str) -> Column {
    Column { name, definition: "{integer} NOT NULL DEFAULT 0", copy: CopyRule::Boolean }
}

pub(super) const fn required_reference(name: &'static str, parent: &'static str, definition: &'static str) -> Column {
    Column { name, definition, copy: CopyRule::RequiredReference(parent) }
}

pub(super) const fn optional_reference(name: &'static str, parent: &'static str, definition: &'static str) -> Column {
    Column { name, definition, copy: CopyRule::OptionalReference(parent) }
}

const TABLE_GROUPS: &[&[Table]] = &[accounts::TABLES, content::TABLES, sharing::TABLES, api::TABLES];

pub fn tables() -> impl Iterator<Item = &'static Table> {
    TABLE_GROUPS.iter().flat_map(|group| group.iter())
}

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
    "CREATE INDEX IF NOT EXISTS idx_comments_parent ON comments (parent_id)",
    "CREATE INDEX IF NOT EXISTS idx_document_suggestions_document ON document_suggestions (document_id)",
    "CREATE INDEX IF NOT EXISTS idx_api_keys_user ON api_keys (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_projects_owner_folder ON projects (owner_id, folder_id)",
    "CREATE INDEX IF NOT EXISTS idx_projects_folder ON projects (folder_id)",
    "CREATE INDEX IF NOT EXISTS idx_project_collaborators_user ON project_collaborators (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_folder_collaborators_user ON folder_collaborators (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_document_shared_files_file ON document_shared_files (file_id)",
    "CREATE INDEX IF NOT EXISTS idx_ownership_transfers_to_user ON ownership_transfers (to_user_id)",
    "CREATE INDEX IF NOT EXISTS idx_packages_owner ON packages (owner_id)",
    "CREATE INDEX IF NOT EXISTS idx_device_tokens_user ON device_tokens (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_user_sessions_user ON user_sessions (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_email_tokens_user ON email_tokens (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_user_identities_user ON user_identities (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_audit_log_created ON audit_log (created_at)",
    "CREATE INDEX IF NOT EXISTS idx_team_members_user ON team_members (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_document_versions_document_created ON document_versions (document_id, created_at)",
    "CREATE INDEX IF NOT EXISTS idx_ownership_transfers_from_user ON ownership_transfers (from_user_id)",
    "CREATE INDEX IF NOT EXISTS idx_document_stars_document ON document_stars (document_id)",
    "CREATE INDEX IF NOT EXISTS idx_project_stars_project ON project_stars (project_id)",
    "CREATE INDEX IF NOT EXISTS idx_desktop_link_codes_user ON desktop_link_codes (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_package_members_user ON package_members (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_package_teams_team ON package_teams (team_id)",
    "CREATE INDEX IF NOT EXISTS idx_teams_created_by ON teams (created_by)",
    "CREATE INDEX IF NOT EXISTS idx_api_render_cache_created ON api_render_cache (created_at)",
    "CREATE INDEX IF NOT EXISTS idx_api_key_usage_detail_minute ON api_key_usage_detail (minute)",
];

pub struct Backfill {
    pub since_version: i64,
    pub statement: &'static str,
}

pub const BACKFILLS: &[Backfill] = &[
    Backfill { since_version: 7, statement: "UPDATE files SET size_bytes = LENGTH(data) WHERE storage_key IS NULL" },
    Backfill { since_version: 8, statement: "UPDATE users SET email_verified = 1" },
    Backfill { since_version: 8, statement: "UPDATE api_keys SET rate_limit = 0" },
];

pub const RETIRED_TABLES: &[&str] = &["document_history", "invitations"];

pub const LEGACY_RENAMES: &[&str] = &[
    "ALTER TABLE spaces RENAME TO projects",
    "ALTER TABLE space_files RENAME TO project_files",
    "ALTER TABLE space_collaborators RENAME TO project_collaborators",
    "ALTER TABLE project_files RENAME COLUMN space_id TO project_id",
    "ALTER TABLE project_collaborators RENAME COLUMN space_id TO project_id",
];

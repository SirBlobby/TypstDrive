use super::{Table, plain, created_at, optional_reference};

pub const TABLES: &[Table] = &[Table {
    name: "themes",
    columns: &[
        plain("id", "TEXT PRIMARY KEY"),
        optional_reference("owner_id", "users", "TEXT REFERENCES users(id) ON DELETE CASCADE"),
        plain("name", "TEXT NOT NULL"),
        plain("palette", "TEXT NOT NULL"),
        created_at("created_at"),
        created_at("updated_at"),
    ],
    constraints: &[],
}];

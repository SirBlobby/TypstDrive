use super::{Table, plain, created_at, required_reference, optional_reference};

pub const TABLES: &[Table] = &[Table {
    name: "trash_items",
    columns: &[
        plain("id", "TEXT PRIMARY KEY"),
        required_reference("owner_id", "users", "TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE"),
        plain("kind", "TEXT NOT NULL"),
        plain("item_id", "TEXT NOT NULL"),
        plain("name", "TEXT NOT NULL"),
        plain("original_folder_id", "TEXT"),
        optional_reference("deleted_by", "users", "TEXT REFERENCES users(id) ON DELETE SET NULL"),
        created_at("deleted_at"),
    ],
    constraints: &[],
}];

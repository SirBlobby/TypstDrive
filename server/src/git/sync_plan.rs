use std::collections::{BTreeMap, HashMap};

use serde::Serialize;

use super::file_rules::FileRules;
use super::local_files::LocalFile;
use super::remote::{ChangeKind, RemoteEntry};
use super::snapshot::Snapshot;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Change {
    pub path: String,
    pub change: &'static str,
}

pub fn local_changes(local: &HashMap<String, LocalFile>, snapshot: &Snapshot, rules: &FileRules) -> BTreeMap<String, ChangeKind> {
    let mut changes = BTreeMap::new();
    for (path, file) in local {
        match snapshot.get(path) {
            None => {
                changes.insert(path.clone(), ChangeKind::Added);
            }
            Some(entry) if entry.content_hash != file.hash => {
                changes.insert(path.clone(), ChangeKind::Modified);
            }
            Some(_) => {}
        }
    }
    for path in snapshot.keys().filter(|path| rules.is_managed(path) && !local.contains_key(*path)) {
        changes.insert(path.clone(), ChangeKind::Deleted);
    }
    changes
}

pub fn remote_changes(entries: &[RemoteEntry], snapshot: &Snapshot, rules: &FileRules) -> BTreeMap<String, ChangeKind> {
    let mut changes = BTreeMap::new();
    for entry in entries.iter().filter(|entry| rules.is_managed(&entry.path)) {
        match snapshot.get(&entry.path) {
            None => {
                changes.insert(entry.path.clone(), ChangeKind::Added);
            }
            Some(saved) if saved.blob_sha != entry.sha => {
                changes.insert(entry.path.clone(), ChangeKind::Modified);
            }
            Some(_) => {}
        }
    }
    let remote_paths: std::collections::HashSet<&str> = entries.iter().map(|entry| entry.path.as_str()).collect();
    for path in snapshot.keys().filter(|path| rules.is_managed(path) && !remote_paths.contains(path.as_str())) {
        changes.insert(path.clone(), ChangeKind::Deleted);
    }
    changes
}

pub fn as_list(changes: &BTreeMap<String, ChangeKind>) -> Vec<Change> {
    changes.iter().map(|(path, kind)| Change { path: path.clone(), change: kind.label() }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::local_files::content_hash;
    use crate::git::snapshot::SnapshotEntry;

    fn local_file(text: &str) -> LocalFile {
        LocalFile { content: text.as_bytes().to_vec(), hash: content_hash(text.as_bytes()), stored_size: text.len() as i64 }
    }

    fn saved(sha: &str, text: &str) -> SnapshotEntry {
        SnapshotEntry { blob_sha: sha.to_string(), content_hash: content_hash(text.as_bytes()) }
    }

    fn entry(path: &str, sha: &str) -> RemoteEntry {
        RemoteEntry { path: path.to_string(), sha: sha.to_string(), size: Some(1) }
    }

    #[test]
    fn finds_local_changes() {
        let rules = FileRules::default();
        let snapshot: Snapshot = [("main.typ".to_string(), saved("a", "old")), ("gone.typ".to_string(), saved("b", "x")), ("same.typ".to_string(), saved("c", "same"))].into();
        let local: HashMap<String, LocalFile> = [("main.typ".to_string(), local_file("new")), ("new.typ".to_string(), local_file("hi")), ("same.typ".to_string(), local_file("same"))].into();
        let changes = local_changes(&local, &snapshot, &rules);
        assert_eq!(changes.get("main.typ"), Some(&ChangeKind::Modified));
        assert_eq!(changes.get("new.typ"), Some(&ChangeKind::Added));
        assert_eq!(changes.get("gone.typ"), Some(&ChangeKind::Deleted));
        assert_eq!(changes.get("same.typ"), None);
    }

    #[test]
    fn finds_remote_changes_and_ignores_unmanaged_files() {
        let rules = FileRules::new(vec!["data/".to_string()]);
        let snapshot: Snapshot = [("main.typ".to_string(), saved("a", "x")), ("old.typ".to_string(), saved("b", "x")), ("data/t.csv".to_string(), saved("c", "x"))].into();
        let entries = vec![entry("main.typ", "a2"), entry("added.bib", "d"), entry("script.py", "e")];
        let changes = remote_changes(&entries, &snapshot, &rules);
        assert_eq!(changes.get("main.typ"), Some(&ChangeKind::Modified));
        assert_eq!(changes.get("added.bib"), Some(&ChangeKind::Added));
        assert_eq!(changes.get("old.typ"), Some(&ChangeKind::Deleted));
        assert_eq!(changes.get("script.py"), None);
        assert_eq!(changes.get("data/t.csv"), None);
    }
}

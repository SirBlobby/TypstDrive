use crate::drive::projects::is_text_path;

pub const MAX_FILE_BYTES: u64 = 50 * 1024 * 1024;

const ASSET_EXTENSIONS: &[&str] = &[".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp", ".pdf", ".ttf", ".otf", ".woff", ".woff2"];

#[derive(Clone, Default)]
pub struct FileRules {
    excluded: Vec<String>,
}

impl FileRules {
    pub fn new(excluded: Vec<String>) -> Self {
        Self { excluded }
    }

    fn is_excluded(&self, path: &str) -> bool {
        let lower = path.to_ascii_lowercase();
        self.excluded.iter().any(|rule| match rule.strip_prefix('*') {
            Some(suffix) => lower.ends_with(&suffix.to_ascii_lowercase()),
            None => {
                let folder = rule.trim_end_matches('/');
                path == folder || path.starts_with(&format!("{folder}/"))
            }
        })
    }

    pub fn is_managed(&self, path: &str) -> bool {
        let lower = path.to_ascii_lowercase();
        let is_document_file = is_text_path(path) || ASSET_EXTENSIONS.iter().any(|extension| lower.ends_with(extension));
        is_document_file && !self.is_excluded(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manages_document_files_only() {
        let rules = FileRules::default();
        assert!(rules.is_managed("main.typ"));
        assert!(rules.is_managed("figures/plot.png"));
        assert!(rules.is_managed("fonts/Inter.woff2"));
        assert!(!rules.is_managed("data/raw.parquet"));
        assert!(!rules.is_managed("build.sh"));
    }

    #[test]
    fn applies_exclusions() {
        let rules = FileRules::new(vec!["data/".to_string(), "*.csv".to_string(), "notes.md".to_string()]);
        assert!(!rules.is_managed("data/table.json"));
        assert!(!rules.is_managed("results/summary.CSV"));
        assert!(!rules.is_managed("notes.md"));
        assert!(rules.is_managed("database/main.typ"));
        assert!(rules.is_managed("main.typ"));
    }
}

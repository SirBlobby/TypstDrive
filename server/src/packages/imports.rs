use std::collections::{HashMap, HashSet};

use crate::compile::compiler::ProjectInput;
use crate::db::sql;
use crate::AppState;

const IMPORT_PREFIX: &str = "@typstdrive/";
const MAX_PACKAGES: usize = 50;

pub type PackageFiles = HashMap<String, HashMap<String, Vec<u8>>>;

fn is_package_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | ':')
}

pub fn imported_packages(source: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for (start, _) in source.match_indices(IMPORT_PREFIX) {
        let rest = &source[start + IMPORT_PREFIX.len()..];
        let end = rest.find(|character: char| !is_package_character(character)).unwrap_or(rest.len());
        if let Some((name, version)) = rest[..end].split_once(':') {
            if !name.is_empty() && !version.is_empty() {
                found.push((name.to_string(), version.to_string()));
            }
        }
    }
    found
}

fn typst_sources<'a>(files: impl Iterator<Item = (&'a String, &'a Vec<u8>)>) -> Vec<&'a str> {
    files
        .filter(|(path, _)| path.ends_with(".typ"))
        .filter_map(|(_, data)| std::str::from_utf8(data).ok())
        .collect()
}

async fn load_version(state: &AppState, account_id: &str, name: &str, version: &str) -> Option<HashMap<String, Vec<u8>>> {
    let rows = sqlx::query_as::<_, (String, Vec<u8>)>(sql(
        "SELECT f.path, f.data \
         FROM package_files f \
         JOIN package_versions v ON v.id = f.version_id \
         JOIN packages p ON p.id = v.package_id \
         WHERE p.namespace = 'typstdrive' AND p.name = $1 AND v.version = $2 \
         AND (p.visibility = 'public' OR p.owner_id = $3 \
         OR EXISTS (SELECT 1 FROM package_members m WHERE m.package_id = p.id AND m.user_id = $4) \
         OR EXISTS (SELECT 1 FROM package_teams t WHERE t.package_id = p.id AND (t.team_id = $5 \
         OR t.team_id IN (SELECT tm.team_id FROM team_members tm WHERE tm.user_id = $6))))",
    ))
    .bind(name)
    .bind(version)
    .bind(account_id)
    .bind(account_id)
    .bind(account_id)
    .bind(account_id)
    .fetch_all(&state.db)
    .await
    .ok()?;
    (!rows.is_empty()).then(|| rows.into_iter().collect())
}

pub async fn load_imported(state: &AppState, account_id: &str, sources: &[&str]) -> PackageFiles {
    let mut packages = PackageFiles::new();
    let mut seen = HashSet::new();
    let mut pending: Vec<(String, String)> = sources.iter().flat_map(|source| imported_packages(source)).collect();
    while let Some((name, version)) = pending.pop() {
        if seen.len() >= MAX_PACKAGES || !seen.insert((name.clone(), version.clone())) {
            continue;
        }
        let Some(files) = load_version(state, account_id, &name, &version).await else {
            continue;
        };
        for source in typst_sources(files.iter()) {
            pending.extend(imported_packages(source));
        }
        packages.insert(format!("{name}:{version}"), files);
    }
    packages
}

pub fn reader_account(owner_id: &str, role: crate::sharing::access::AccessRole, user_id: &Option<String>) -> String {
    if role == crate::sharing::access::AccessRole::Owner {
        owner_id.to_string()
    } else {
        user_id.clone().unwrap_or_default()
    }
}

pub async fn attach_imported(state: &AppState, input: &mut ProjectInput, account_id: &str) {
    let sources = typst_sources(input.files.iter());
    let packages = load_imported(state, account_id, &sources).await;
    input.packages.extend(packages);
}

#[cfg(test)]
mod tests {
    use super::imported_packages;

    #[test]
    fn finds_typstdrive_imports() {
        let source = "#import \"@typstdrive/my-package:0.0.1\": *\n#import \"@preview/cetz:0.3.0\"\n#include \"@typstdrive/other:1.2.0/lib.typ\"";
        assert_eq!(
            imported_packages(source),
            vec![("my-package".to_string(), "0.0.1".to_string()), ("other".to_string(), "1.2.0".to_string())]
        );
    }

    #[test]
    fn ignores_incomplete_specs() {
        assert!(imported_packages("@typstdrive/ @typstdrive/name @typstdrive/:1.0.0").is_empty());
    }
}

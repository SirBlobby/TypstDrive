mod cli;
mod github;
mod gitlab;
mod http;

use axum::http::StatusCode;
use serde::Serialize;

use super::repo_settings::RepoUrl;
use super::ApiError;
use crate::drive::project_paths::clean_project_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Github,
    Gitlab,
    Generic,
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Github => "github",
            Self::Gitlab => "gitlab",
            Self::Generic => "generic",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "github" => Some(Self::Github),
            "gitlab" => Some(Self::Gitlab),
            "generic" => Some(Self::Generic),
            _ => None,
        }
    }

    pub fn detect(url: &RepoUrl) -> Self {
        match url.host_name() {
            "github.com" | "www.github.com" => Self::Github,
            "gitlab.com" | "www.gitlab.com" => Self::Gitlab,
            _ => Self::Generic,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Github => "GitHub",
            Self::Gitlab => "GitLab",
            Self::Generic => "The Git server",
        }
    }
}

#[derive(Clone)]
pub struct RemoteRepo {
    pub provider: Provider,
    pub url: RepoUrl,
    pub branch: String,
    pub folder: String,
    pub username: Option<String>,
    pub token: Option<String>,
    pub cache_name: String,
}

impl RemoteRepo {
    pub fn full_path(&self, path: &str) -> String {
        if self.folder.is_empty() {
            path.to_string()
        } else {
            format!("{}/{path}", self.folder)
        }
    }

    pub fn relative_path(&self, full_path: &str) -> Option<String> {
        if self.folder.is_empty() {
            return Some(full_path.to_string());
        }
        full_path.strip_prefix(&format!("{}/", self.folder)).map(str::to_string)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoteEntry {
    pub path: String,
    pub sha: String,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
}

impl ChangeKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Modified => "modified",
            Self::Deleted => "deleted",
        }
    }
}

pub struct FileChange {
    pub path: String,
    pub kind: ChangeKind,
    pub content: Vec<u8>,
}

#[derive(Clone)]
pub struct Author {
    pub name: String,
    pub email: String,
}

async fn blocking<T: Send + 'static>(work: impl FnOnce() -> Result<T, ApiError> + Send + 'static) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
}

fn is_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn checked_object_id(provider: Provider, value: String) -> Result<String, ApiError> {
    if is_object_id(&value) {
        Ok(value.to_ascii_lowercase())
    } else {
        Err((StatusCode::BAD_GATEWAY, format!("{} sent an unexpected commit id", provider.label())))
    }
}

pub async fn head(repo: &RemoteRepo) -> Result<String, ApiError> {
    let repo = repo.clone();
    let provider = repo.provider;
    let commit = blocking(move || match repo.provider {
        Provider::Github => github::head(&repo),
        Provider::Gitlab => gitlab::head(&repo),
        Provider::Generic => cli::head(&repo),
    })
    .await?;
    checked_object_id(provider, commit)
}

pub async fn list_files(repo: &RemoteRepo, commit: &str) -> Result<Vec<RemoteEntry>, ApiError> {
    let repo = repo.clone();
    let commit = commit.to_string();
    let entries = blocking(move || match repo.provider {
        Provider::Github => github::list_files(&repo, &commit),
        Provider::Gitlab => gitlab::list_files(&repo, &commit),
        Provider::Generic => cli::list_files(&repo, &commit),
    })
    .await?;
    Ok(entries.into_iter().filter(is_safe_entry).collect())
}

fn is_safe_entry(entry: &RemoteEntry) -> bool {
    is_object_id(&entry.sha) && clean_project_path(&entry.path).is_ok_and(|cleaned| cleaned == entry.path)
}

pub fn download_concurrency(provider: Provider) -> usize {
    match provider {
        Provider::Generic => 1,
        Provider::Github | Provider::Gitlab => 6,
    }
}

pub async fn prefetch(repo: &RemoteRepo, entries: &[RemoteEntry]) {
    if repo.provider != Provider::Generic || entries.is_empty() {
        return;
    }
    let repo = repo.clone();
    let shas: Vec<String> = entries.iter().map(|entry| entry.sha.clone()).collect();
    let _ = blocking(move || cli::prefetch(&repo, &shas)).await;
}

pub async fn fill_sizes(repo: &RemoteRepo, commit: &str, entries: Vec<RemoteEntry>) -> Result<Vec<RemoteEntry>, ApiError> {
    if repo.provider != Provider::Gitlab || entries.iter().all(|entry| entry.size.is_some()) {
        return Ok(entries);
    }
    let repo = repo.clone();
    let commit = commit.to_string();
    blocking(move || gitlab::fill_sizes(&repo, &commit, entries)).await
}

pub async fn read_blob(repo: &RemoteRepo, entry: &RemoteEntry) -> Result<Option<Vec<u8>>, ApiError> {
    let repo = repo.clone();
    let sha = entry.sha.clone();
    blocking(move || match repo.provider {
        Provider::Github => github::read_blob(&repo, &sha),
        Provider::Gitlab => gitlab::read_blob(&repo, &sha),
        Provider::Generic => cli::read_blob(&repo, &sha),
    })
    .await
}

pub async fn commit(repo: &RemoteRepo, base_commit: &str, changes: Vec<FileChange>, message: &str, author: &Author) -> Result<String, ApiError> {
    let repo = repo.clone();
    let base_commit = base_commit.to_string();
    let message = message.to_string();
    let author = author.clone();
    let provider = repo.provider;
    let commit = blocking(move || match repo.provider {
        Provider::Github => github::commit(&repo, &base_commit, &changes, &message, &author),
        Provider::Gitlab => gitlab::commit(&repo, &changes, &message, &author),
        Provider::Generic => cli::commit(&repo, &base_commit, &changes, &message, &author),
    })
    .await?;
    checked_object_id(provider, commit)
}

pub async fn token_owner(provider: Provider, server: &RepoUrl, token: &str) -> Result<String, ApiError> {
    let server = server.clone();
    let token = token.to_string();
    blocking(move || match provider {
        Provider::Github => github::token_owner(&server, &token),
        Provider::Gitlab => gitlab::token_owner(&server, &token),
        Provider::Generic => Err((StatusCode::BAD_REQUEST, "Saved accounts are only for GitHub and GitLab".to_string())),
    })
    .await
}

pub fn forget_cache(cache_name: &str) {
    cli::remove_cache(cache_name);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(folder: &str) -> RemoteRepo {
        RemoteRepo {
            provider: Provider::Github,
            url: RepoUrl::parse("https://github.com/owner/repo").unwrap(),
            branch: "main".to_string(),
            folder: folder.to_string(),
            username: None,
            token: None,
            cache_name: "test".to_string(),
        }
    }

    #[test]
    fn maps_paths_inside_the_folder() {
        let paper = repo("paper");
        assert_eq!(paper.full_path("main.typ"), "paper/main.typ");
        assert_eq!(paper.relative_path("paper/figures/a.png").as_deref(), Some("figures/a.png"));
        assert_eq!(paper.relative_path("papers/main.typ"), None);
        assert_eq!(repo("").relative_path("main.typ").as_deref(), Some("main.typ"));
    }

    #[test]
    fn rejects_unsafe_remote_entries() {
        let sha = "a".repeat(40);
        let entry = |path: &str, sha: &str| RemoteEntry { path: path.to_string(), sha: sha.to_string(), size: None };
        assert!(is_safe_entry(&entry("chapters/intro.typ", &sha)));
        assert!(!is_safe_entry(&entry("../escape.typ", &sha)));
        assert!(!is_safe_entry(&entry("/absolute.typ", &sha)));
        assert!(!is_safe_entry(&entry("a//b.typ", &sha)));
        assert!(!is_safe_entry(&entry("dir\\file.typ", &sha)));
        assert!(!is_safe_entry(&entry("main.typ", "not-a-sha")));
    }

    #[test]
    fn detects_hosted_providers() {
        assert_eq!(Provider::detect(&RepoUrl::parse("https://github.com/a/b").unwrap()), Provider::Github);
        assert_eq!(Provider::detect(&RepoUrl::parse("https://gitlab.com/a/b/c").unwrap()), Provider::Gitlab);
        assert_eq!(Provider::detect(&RepoUrl::parse("https://codeberg.org/a/b").unwrap()), Provider::Generic);
    }
}

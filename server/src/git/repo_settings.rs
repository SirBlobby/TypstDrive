use axum::http::StatusCode;

use super::ApiError;
use crate::drive::project_paths::clean_project_path;

const MAX_BRANCH_LENGTH: usize = 200;
const MAX_EXCLUDED_PATHS: usize = 100;
const MAX_EXCLUDED_PATH_LENGTH: usize = 512;
const FORBIDDEN_BRANCH_CHARACTERS: &[char] = &[' ', '~', '^', ':', '?', '*', '[', '\\'];

fn invalid(message: &str) -> ApiError {
    (StatusCode::BAD_REQUEST, message.to_string())
}

#[derive(Clone)]
pub struct RepoUrl {
    pub scheme: String,
    pub authority: String,
    pub path: String,
    pub text: String,
}

fn is_safe_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && segment.chars().all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
}

fn is_safe_authority(authority: &str) -> bool {
    !authority.is_empty()
        && !authority.starts_with(':')
        && authority.chars().all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '.' | ':' | '[' | ']'))
}

impl RepoUrl {
    pub fn parse(input: &str) -> Result<Self, ApiError> {
        let trimmed = input.trim().trim_end_matches('/');
        let (scheme, rest) = if let Some(rest) = trimmed.strip_prefix("https://") {
            ("https", rest)
        } else if let Some(rest) = trimmed.strip_prefix("http://") {
            ("http", rest)
        } else {
            return Err(invalid("Use the https:// address of the repository"));
        };

        let (authority, full_path) = rest.split_once('/').ok_or_else(|| invalid("The address needs the owner and repository name"))?;
        if authority.contains('@') {
            return Err(invalid("Put the access token in the token field instead of the address"));
        }
        if !is_safe_authority(authority) {
            return Err(invalid("The repository address has an unsupported host name"));
        }

        let path = full_path.strip_suffix(".git").unwrap_or(full_path);
        let segments: Vec<&str> = path.split('/').collect();
        if segments.len() < 2 || !segments.iter().all(|segment| is_safe_segment(segment)) {
            return Err(invalid("The address needs the owner and repository name, like https://github.com/owner/repository"));
        }

        Ok(Self {
            scheme: scheme.to_string(),
            authority: authority.to_ascii_lowercase(),
            path: path.to_string(),
            text: format!("{scheme}://{}/{full_path}", authority.to_ascii_lowercase()),
        })
    }

    pub fn parse_server(input: &str) -> Result<Self, ApiError> {
        let trimmed = input.trim().trim_end_matches('/');
        let with_scheme = if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
            trimmed.to_string()
        } else {
            format!("https://{trimmed}")
        };
        let (scheme, authority) = with_scheme.split_once("://").unwrap_or(("https", ""));
        if authority.contains('/') || authority.contains('@') || !is_safe_authority(authority) {
            return Err(invalid("Enter only the server address, like gitlab.example.com"));
        }
        let authority = authority.to_ascii_lowercase();
        Ok(Self { scheme: scheme.to_string(), text: format!("{scheme}://{authority}"), authority, path: String::new() })
    }

    pub fn host_name(&self) -> &str {
        match self.authority.rsplit_once(':') {
            Some((host, port)) if !host.is_empty() && port.chars().all(|character| character.is_ascii_digit()) && !self.authority.ends_with(']') => host,
            _ => &self.authority,
        }
    }

    pub fn host_and_port(&self) -> String {
        if self.host_name() != self.authority {
            return self.authority.clone();
        }
        let port = if self.scheme == "https" { 443 } else { 80 };
        format!("{}:{port}", self.authority)
    }

    pub fn web_base(&self) -> String {
        format!("{}://{}", self.scheme, self.authority)
    }
}

pub fn clean_branch(branch: &str) -> Result<String, ApiError> {
    let branch = branch.trim();
    let is_valid = !branch.is_empty()
        && branch.len() <= MAX_BRANCH_LENGTH
        && !branch.starts_with(['-', '/', '.'])
        && !branch.ends_with(['/', '.'])
        && !branch.ends_with(".lock")
        && !branch.contains("..")
        && !branch.contains("//")
        && !branch.contains("@{")
        && !branch.chars().any(|character| character.is_control() || FORBIDDEN_BRANCH_CHARACTERS.contains(&character));
    if is_valid {
        Ok(branch.to_string())
    } else {
        Err(invalid("That is not a valid branch name"))
    }
}

pub fn clean_folder(folder: &str) -> Result<String, ApiError> {
    let folder = folder.trim().trim_matches('/');
    if folder.is_empty() {
        return Ok(String::new());
    }
    clean_project_path(folder)
}

pub fn clean_excluded_paths(paths: &[String]) -> Result<Vec<String>, ApiError> {
    let cleaned: Vec<String> = paths
        .iter()
        .map(|path| path.trim().trim_start_matches("./").trim_start_matches('/').to_string())
        .filter(|path| !path.is_empty())
        .collect();
    if cleaned.len() > MAX_EXCLUDED_PATHS {
        return Err(invalid("Too many excluded paths"));
    }
    if cleaned.iter().any(|path| path.len() > MAX_EXCLUDED_PATH_LENGTH || path.chars().any(char::is_control)) {
        return Err(invalid("Excluded paths must be short and on one line each"));
    }
    Ok(cleaned)
}

pub fn clean_token(token: Option<&str>) -> Option<String> {
    token.map(str::trim).filter(|token| !token.is_empty()).map(str::to_string)
}

pub fn clean_username(username: Option<&str>) -> Option<String> {
    username
        .map(str::trim)
        .filter(|username| !username.is_empty())
        .map(|username| username.chars().filter(|character| !character.is_control() && *character != ':').take(100).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_repository_addresses() {
        let url = RepoUrl::parse("https://GitHub.com/owner/repo.git/").unwrap();
        assert_eq!(url.authority, "github.com");
        assert_eq!(url.path, "owner/repo");
        assert_eq!(url.host_and_port(), "github.com:443");

        let nested = RepoUrl::parse("http://git.example.com:3000/group/sub/repo").unwrap();
        assert_eq!(nested.host_name(), "git.example.com");
        assert_eq!(nested.host_and_port(), "git.example.com:3000");
        assert_eq!(nested.path, "group/sub/repo");
    }

    #[test]
    fn rejects_unsafe_addresses() {
        assert!(RepoUrl::parse("ssh://github.com/owner/repo").is_err());
        assert!(RepoUrl::parse("https://user:token@github.com/owner/repo").is_err());
        assert!(RepoUrl::parse("https://github.com/owner").is_err());
        assert!(RepoUrl::parse("https://github.com/owner/../repo").is_err());
        assert!(RepoUrl::parse("https://github.com/owner/repo?x=1").is_err());
    }

    #[test]
    fn parses_server_addresses() {
        assert_eq!(RepoUrl::parse_server("GitLab.example.com/").unwrap().web_base(), "https://gitlab.example.com");
        assert_eq!(RepoUrl::parse_server("http://git.local:3000").unwrap().web_base(), "http://git.local:3000");
        assert!(RepoUrl::parse_server("https://gitlab.com/group").is_err());
        assert!(RepoUrl::parse_server("https://user@gitlab.com").is_err());
    }

    #[test]
    fn validates_branches_and_folders() {
        assert_eq!(clean_branch(" feature/paper ").unwrap(), "feature/paper");
        assert!(clean_branch("-bad").is_err());
        assert!(clean_branch("a..b").is_err());
        assert!(clean_branch("has space").is_err());
        assert_eq!(clean_folder("/paper/").unwrap(), "paper");
        assert_eq!(clean_folder("").unwrap(), "");
        assert!(clean_folder("../secret").is_err());
    }
}

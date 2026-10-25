use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use axum::http::StatusCode;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

use super::{Author, ChangeKind, FileChange, RemoteEntry, RemoteRepo};
use crate::git::file_rules::MAX_FILE_BYTES;
use crate::git::ApiError;
use crate::remote_hosts::resolve_remote_host;

const INDEX_FILE_NAME: &str = "typstdrive-index";
const FILE_MODE: &str = "100644";
const SYMLINK_MODE: &str = "120000";
const REMOVED_ENTRY: &str = "0 0000000000000000000000000000000000000000";

fn cache_root() -> PathBuf {
    std::env::var("GIT_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("typstdrive-git"))
}

fn repo_dir(repo: &RemoteRepo) -> PathBuf {
    cache_root().join(&repo.cache_name)
}

pub fn remove_cache(cache_name: &str) {
    let _ = std::fs::remove_dir_all(cache_root().join(cache_name));
}

fn check_host(repo: &RemoteRepo) -> Result<(), ApiError> {
    resolve_remote_host(&repo.url.host_and_port()).map(|_| ()).map_err(|error| match error.kind() {
        ErrorKind::PermissionDenied => (StatusCode::BAD_REQUEST, "The repository must be on a public address".to_string()),
        _ => (StatusCode::BAD_GATEWAY, format!("Could not find the Git server {}", repo.url.host_name())),
    })
}

fn null_device() -> &'static str {
    if cfg!(windows) { "NUL" } else { "/dev/null" }
}

fn config_entries(repo: &RemoteRepo) -> Vec<(String, String)> {
    let mut entries = vec![
        ("protocol.allow".to_string(), "never".to_string()),
        (format!("protocol.{}.allow", repo.url.scheme), "always".to_string()),
        ("credential.helper".to_string(), String::new()),
        ("http.followRedirects".to_string(), "false".to_string()),
        ("http.lowSpeedLimit".to_string(), "1000".to_string()),
        ("http.lowSpeedTime".to_string(), "60".to_string()),
    ];
    if let Some(token) = &repo.token {
        let username = repo.username.as_deref().unwrap_or("git");
        let credentials = BASE64.encode(format!("{username}:{token}"));
        entries.push(("http.extraHeader".to_string(), format!("Authorization: Basic {credentials}")));
    }
    entries
}

fn git(repo: &RemoteRepo, git_dir: Option<&Path>) -> Command {
    let mut command = Command::new("git");
    if let Some(git_dir) = git_dir {
        command.arg("--git-dir").arg(git_dir);
    }
    let entries = config_entries(repo);
    command
        .env_remove("GIT_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", null_device())
        .env("GIT_CONFIG_COUNT", entries.len().to_string());
    for (index, (key, value)) in entries.into_iter().enumerate() {
        command.env(format!("GIT_CONFIG_KEY_{index}"), key).env(format!("GIT_CONFIG_VALUE_{index}"), value);
    }
    command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    command
}

fn failure(stderr: &[u8]) -> ApiError {
    let text = String::from_utf8_lossy(stderr);
    let lower = text.to_lowercase();
    let mentions = |needles: &[&str]| needles.iter().any(|needle| lower.contains(needle));
    if mentions(&["authentication failed", "could not read username", "could not read password", "403", "401", "access denied"]) {
        return (StatusCode::FORBIDDEN, "The Git server did not accept the access token. It needs read and write access to this repository.".to_string());
    }
    if mentions(&["non-fast-forward", "fetch first", "[rejected]"]) {
        return (StatusCode::CONFLICT, "The repository has new commits. Pull them first.".to_string());
    }
    if mentions(&["couldn't find remote ref", "not found", "does not appear to be a git repository"]) {
        return (StatusCode::NOT_FOUND, "The Git server could not find the repository or branch. Private repositories need an access token.".to_string());
    }
    let last_line = text.lines().rev().find(|line| !line.trim().is_empty()).unwrap_or("unknown error");
    (StatusCode::BAD_GATEWAY, format!("Git failed: {}", last_line.trim()))
}

fn spawn_error(error: std::io::Error) -> ApiError {
    match error.kind() {
        ErrorKind::NotFound => (StatusCode::INTERNAL_SERVER_ERROR, "Git is not installed on this server".to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

fn run(mut command: Command) -> Result<String, ApiError> {
    let output = command.output().map_err(spawn_error)?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(failure(&output.stderr))
    }
}

fn run_with_input(mut command: Command, input: &[u8]) -> Result<String, ApiError> {
    let mut child = command.stdin(Stdio::piped()).spawn().map_err(spawn_error)?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input).map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    }
    let output = child.wait_with_output().map_err(spawn_error)?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(failure(&output.stderr))
    }
}

fn branch_ref(repo: &RemoteRepo) -> String {
    format!("refs/heads/{}", repo.branch)
}

fn fetch(repo: &RemoteRepo) -> Result<PathBuf, ApiError> {
    check_host(repo)?;
    let dir = repo_dir(repo);
    if dir.join("HEAD").exists() {
        let mut set_url = git(repo, Some(&dir));
        set_url.args(["remote", "set-url", "origin", repo.url.text.as_str()]);
        run(set_url)?;
        let mut fetch = git(repo, Some(&dir));
        fetch.args(["fetch", "--depth=1", "--filter=blob:none", "--no-tags", "--update-head-ok", "origin"]);
        fetch.arg(format!("+{0}:{0}", branch_ref(repo)));
        run(fetch)?;
        return Ok(dir);
    }

    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(cache_root()).map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    let mut clone = git(repo, None);
    clone.args(["clone", "--bare", "--depth=1", "--filter=blob:none", "--single-branch", "--no-tags", "--branch", repo.branch.as_str(), "--", repo.url.text.as_str()]);
    clone.arg(&dir);
    if let Err(error) = run(clone) {
        let _ = std::fs::remove_dir_all(&dir);
        return Err(error);
    }
    Ok(dir)
}

fn has_commit(repo: &RemoteRepo, dir: &Path, commit: &str) -> bool {
    let mut check = git(repo, Some(dir));
    check.args(["cat-file", "-e", format!("{commit}^{{commit}}").as_str()]);
    dir.join("HEAD").exists() && run(check).is_ok()
}

fn dir_with_commit(repo: &RemoteRepo, commit: &str) -> Result<PathBuf, ApiError> {
    let dir = repo_dir(repo);
    if has_commit(repo, &dir, commit) {
        return Ok(dir);
    }
    let dir = fetch(repo)?;
    if has_commit(repo, &dir, commit) {
        Ok(dir)
    } else {
        Err((StatusCode::CONFLICT, "The repository has changed since the last sync. Pull the latest changes first.".to_string()))
    }
}

pub fn head(repo: &RemoteRepo) -> Result<String, ApiError> {
    let dir = fetch(repo)?;
    let mut parse = git(repo, Some(&dir));
    parse.args(["rev-parse", "--verify", format!("{}^{{commit}}", branch_ref(repo)).as_str()]);
    run(parse)
}

pub fn list_files(repo: &RemoteRepo, commit: &str) -> Result<Vec<RemoteEntry>, ApiError> {
    let dir = dir_with_commit(repo, commit)?;
    let mut list = git(repo, Some(&dir));
    list.args(["ls-tree", "-r", "-z", commit]);
    if !repo.folder.is_empty() {
        list.args(["--", format!("{}/", repo.folder).as_str()]);
    }
    let output = run(list)?;
    Ok(output
        .split('\0')
        .filter_map(|line| {
            let (header, full_path) = line.split_once('\t')?;
            let mut fields = header.split(' ');
            let (mode, kind, sha) = (fields.next()?, fields.next()?, fields.next()?);
            (kind == "blob" && mode != SYMLINK_MODE).then_some(())?;
            Some(RemoteEntry { path: repo.relative_path(full_path)?, sha: sha.to_string(), size: None })
        })
        .collect())
}

pub fn read_blob(repo: &RemoteRepo, sha: &str) -> Result<Option<Vec<u8>>, ApiError> {
    check_host(repo)?;
    let mut read = git(repo, Some(&repo_dir(repo)));
    read.args(["cat-file", "blob", sha]);
    let mut child = read.spawn().map_err(spawn_error)?;
    let mut bytes = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        stdout
            .take(MAX_FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    }
    if bytes.len() as u64 > MAX_FILE_BYTES {
        let _ = child.kill();
        let _ = child.wait();
        return Ok(None);
    }
    let output = child.wait_with_output().map_err(spawn_error)?;
    if output.status.success() {
        Ok(Some(bytes))
    } else {
        Err(failure(&output.stderr))
    }
}

fn index_line(repo: &RemoteRepo, dir: &Path, change: &FileChange) -> Result<String, ApiError> {
    let path = repo.full_path(&change.path);
    if change.kind == ChangeKind::Deleted {
        return Ok(format!("{REMOVED_ENTRY}\t{path}\n"));
    }
    let mut hash = git(repo, Some(dir));
    hash.args(["hash-object", "-w", "--stdin"]);
    let sha = run_with_input(hash, &change.content)?;
    Ok(format!("{FILE_MODE} {sha}\t{path}\n"))
}

fn author_command(repo: &RemoteRepo, dir: &Path, author: &Author) -> Command {
    let mut command = git(repo, Some(dir));
    command
        .env("GIT_AUTHOR_NAME", &author.name)
        .env("GIT_AUTHOR_EMAIL", &author.email)
        .env("GIT_COMMITTER_NAME", &author.name)
        .env("GIT_COMMITTER_EMAIL", &author.email);
    command
}

fn write_commit_tree(repo: &RemoteRepo, dir: &Path, index: &Path, base_commit: &str, changes: &[FileChange]) -> Result<String, ApiError> {
    let mut read_tree = git(repo, Some(dir));
    read_tree.env("GIT_INDEX_FILE", index).args(["read-tree", base_commit]);
    run(read_tree)?;

    let lines = changes.iter().map(|change| index_line(repo, dir, change)).collect::<Result<Vec<String>, ApiError>>()?;
    let mut update_index = git(repo, Some(dir));
    update_index.env("GIT_INDEX_FILE", index).args(["update-index", "--index-info"]);
    run_with_input(update_index, lines.concat().as_bytes())?;

    let mut write_tree = git(repo, Some(dir));
    write_tree.env("GIT_INDEX_FILE", index).args(["write-tree", "--missing-ok"]);
    run(write_tree)
}

pub fn commit(repo: &RemoteRepo, base_commit: &str, changes: &[FileChange], message: &str, author: &Author) -> Result<String, ApiError> {
    let dir = dir_with_commit(repo, base_commit)?;
    let index = dir.join(INDEX_FILE_NAME);
    let _ = std::fs::remove_file(&index);
    let tree = write_commit_tree(repo, &dir, &index, base_commit, changes);
    let _ = std::fs::remove_file(&index);
    let tree = tree?;

    let mut commit_tree = author_command(repo, &dir, author);
    commit_tree.args(["commit-tree", tree.as_str(), "-p", base_commit, "-F", "-"]);
    let commit = run_with_input(commit_tree, message.as_bytes())?;

    check_host(repo)?;
    let mut push = git(repo, Some(&dir));
    push.args(["push", "origin", format!("{commit}:{}", branch_ref(repo)).as_str()]);
    run(push)?;

    let mut update_ref = git(repo, Some(&dir));
    update_ref.args(["update-ref", branch_ref(repo).as_str(), commit.as_str()]);
    run(update_ref)?;
    Ok(commit)
}

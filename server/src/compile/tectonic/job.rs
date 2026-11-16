use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Component, Path, PathBuf};

use super::sandbox::{Sandbox, SandboxMode, JAILED_JOBS};
use super::settings::latex_settings;

const SOURCE_FOLDER: &str = "source";
const OUTPUT_FOLDER: &str = "output";

pub struct Job {
    host_folder: PathBuf,
    jailed_folder: String,
}

pub fn safe_relative_path(path: &str) -> Option<PathBuf> {
    if path.is_empty() || path.contains('\0') || path.contains('\\') {
        return None;
    }
    let mut cleaned = PathBuf::new();
    for component in Path::new(path).components() {
        match component {
            Component::Normal(part) => cleaned.push(part),
            Component::CurDir => {}
            _ => return None,
        }
    }
    (!cleaned.as_os_str().is_empty()).then_some(cleaned)
}

fn create_folder(path: &Path, mode: u32) -> std::io::Result<()> {
    fs::DirBuilder::new().recursive(true).mode(mode).create(path)
}

fn write_new_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o644)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    file.write_all(bytes)
}

impl Job {
    pub fn create(sandbox: &Sandbox, files: &HashMap<String, Vec<u8>>) -> std::io::Result<Job> {
        let name = uuid::Uuid::new_v4().simple().to_string();
        let jailed_folder = format!("{JAILED_JOBS}/{name}");
        let host_folder = sandbox.host_path(&jailed_folder);
        fs::DirBuilder::new().mode(0o755).create(&host_folder)?;
        let job = Job { host_folder, jailed_folder };

        let source = job.host_folder.join(SOURCE_FOLDER);
        create_folder(&source, 0o755)?;
        for (path, bytes) in files {
            let Some(relative) = safe_relative_path(path) else { continue };
            let destination = source.join(&relative);
            if let Some(parent) = destination.parent() {
                create_folder(parent, 0o755)?;
            }
            write_new_file(&destination, bytes)?;
        }

        let output = job.host_folder.join(OUTPUT_FOLDER);
        fs::DirBuilder::new().mode(0o700).create(&output)?;
        if sandbox.mode == SandboxMode::Chroot {
            let settings = latex_settings();
            std::os::unix::fs::chown(&output, Some(settings.sandbox_user), Some(settings.sandbox_group))?;
        }
        Ok(job)
    }

    pub fn source_folder(&self, sandbox: &Sandbox) -> String {
        sandbox.visible_path(&format!("{}/{SOURCE_FOLDER}", self.jailed_folder))
    }

    pub fn output_argument(&self) -> String {
        format!("../{OUTPUT_FOLDER}")
    }

    pub fn read_output(&self, file_name: &str) -> std::io::Result<Vec<u8>> {
        let path = self.host_folder.join(OUTPUT_FOLDER).join(file_name);
        let file = fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path)?;
        if !file.metadata()?.is_file() {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }
        let mut bytes = Vec::new();
        file.take(latex_settings().max_file_bytes).read_to_end(&mut bytes)?;
        Ok(bytes)
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.host_folder);
    }
}

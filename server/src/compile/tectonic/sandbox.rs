use std::fs;
use std::io::Read;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use tokio::sync::OnceCell;

use super::launch::{run_tectonic, LaunchPlan};
use super::settings::{latex_settings, SandboxChoice};

pub const JAILED_BINARY: &str = "/bin/tectonic";
pub const JAILED_CA_BUNDLE: &str = "/etc/ssl/certs/ca-certificates.crt";
pub const JAILED_CACHE: &str = "/cache";
pub const JAILED_HOME: &str = "/home";
pub const JAILED_JOBS: &str = "/jobs";

const CA_BUNDLE_CANDIDATES: &[&str] = &[
    "/etc/ssl/certs/ca-certificates.crt",
    "/etc/pki/tls/certs/ca-bundle.crt",
    "/etc/ssl/cert.pem",
    "/etc/ssl/ca-bundle.pem",
];
const NETWORK_FILES: &[&str] = &["/etc/resolv.conf"];
const ELF_MAGIC: &[u8] = b"\x7fELF";
const ELF_INTERPRETER_SEGMENT: u32 = 3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SandboxMode {
    Chroot,
    UserNamespace,
    Unsandboxed,
}

impl SandboxMode {
    pub fn label(self) -> &'static str {
        match self {
            SandboxMode::Chroot => "chroot jail as an unprivileged user",
            SandboxMode::UserNamespace => "chroot jail in a user namespace",
            SandboxMode::Unsandboxed => "not sandboxed",
        }
    }
}

pub struct Sandbox {
    pub root: PathBuf,
    pub mode: SandboxMode,
    pub binary: PathBuf,
    pub version: String,
}

impl Sandbox {
    pub fn is_jailed(&self) -> bool {
        self.mode != SandboxMode::Unsandboxed
    }

    pub fn host_path(&self, jailed_path: &str) -> PathBuf {
        self.root.join(jailed_path.trim_start_matches('/'))
    }

    pub fn visible_path(&self, jailed_path: &str) -> String {
        if self.is_jailed() {
            jailed_path.to_string()
        } else {
            self.host_path(jailed_path).to_string_lossy().into_owned()
        }
    }
}

static SANDBOX: OnceCell<Result<Sandbox, String>> = OnceCell::const_new();

pub async fn sandbox() -> Result<&'static Sandbox, String> {
    SANDBOX
        .get_or_init(|| async {
            let prepared = match tokio::task::spawn_blocking(prepare_files).await {
                Ok(result) => result,
                Err(error) => Err(format!("The LaTeX sandbox could not be prepared: {error}")),
            };
            let checked = match prepared {
                Ok(sandbox) => check_launch(sandbox).await,
                Err(error) => Err(error),
            };
            match &checked {
                Ok(sandbox) => tracing::info!("LaTeX engine ready: {} ({})", sandbox.version, sandbox.mode.label()),
                Err(error) => tracing::error!("LaTeX engine unavailable: {error}"),
            }
            checked
        })
        .await
        .as_ref()
        .map_err(Clone::clone)
}

pub fn warm_up() {
    tokio::spawn(async {
        if sandbox().await.is_err() {
            return;
        }
        let starter = std::collections::HashMap::from([("main.tex".to_string(), crate::compile::latex::LATEX_STARTER.as_bytes().to_vec())]);
        match super::compile_to_pdf("main.tex", starter).await {
            Ok(_) => tracing::info!("LaTeX package cache is ready"),
            Err(errors) => tracing::warn!("LaTeX warm-up compile failed: {}", errors.first().map(|error| error.message.as_str()).unwrap_or("unknown error")),
        }
    });
}

fn io_error(context: &str, error: std::io::Error) -> String {
    format!("{context}: {error}")
}

fn effective_user() -> u32 {
    unsafe { libc::geteuid() }
}

fn choose_mode() -> SandboxMode {
    match latex_settings().sandbox {
        SandboxChoice::Off => SandboxMode::Unsandboxed,
        SandboxChoice::Automatic if effective_user() == 0 => SandboxMode::Chroot,
        SandboxChoice::Automatic => SandboxMode::UserNamespace,
    }
}

fn require_private_directory(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| io_error("Could not read the LaTeX folder", error))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(format!("{} must be a real folder, not a link", path.display()));
    }
    if metadata.uid() != effective_user() {
        return Err(format!("{} must be owned by the user running TypstDrive", path.display()));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).map_err(|error| io_error("Could not set folder permissions", error))
}

fn make_directory(path: &Path, mode: u32) -> Result<(), String> {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(format!("{} must be a real folder", path.display()));
        }
    } else {
        fs::create_dir(path).map_err(|error| io_error(&format!("Could not create {}", path.display()), error))?;
    }
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).map_err(|error| io_error("Could not set folder permissions", error))
}

fn install_file(source: &Path, destination: &Path, mode: u32) -> Result<(), String> {
    let staging = destination.with_extension("installing");
    let _ = fs::remove_file(&staging);
    fs::copy(source, &staging).map_err(|error| io_error(&format!("Could not copy {}", source.display()), error))?;
    fs::set_permissions(&staging, fs::Permissions::from_mode(mode)).map_err(|error| io_error("Could not set file permissions", error))?;
    fs::rename(&staging, destination).map_err(|error| io_error("Could not install a sandbox file", error))
}

fn needs_dynamic_loader(binary: &Path) -> Result<bool, String> {
    let mut header = Vec::new();
    fs::File::open(binary)
        .and_then(|file| file.take(64 * 1024).read_to_end(&mut header))
        .map_err(|error| io_error("Could not read the tectonic program", error))?;
    if !header.starts_with(ELF_MAGIC) || header.len() < 64 || header[4] != 2 || header[5] != 1 {
        return Err("tectonic must be a 64-bit little-endian Linux program".to_string());
    }
    let read_u16 = |offset: usize| u16::from_le_bytes([header[offset], header[offset + 1]]) as usize;
    let header_table = u64::from_le_bytes(header[0x20..0x28].try_into().unwrap_or_default()) as usize;
    let entry_size = read_u16(0x36);
    let entry_count = read_u16(0x38);
    for index in 0..entry_count {
        let offset = header_table + index * entry_size;
        let Some(bytes) = header.get(offset..offset + 4) else { break };
        if u32::from_le_bytes(bytes.try_into().unwrap_or_default()) == ELF_INTERPRETER_SEGMENT {
            return Ok(true);
        }
    }
    Ok(false)
}

fn find_ca_bundle() -> Option<PathBuf> {
    let from_environment = std::env::var_os("SSL_CERT_FILE").map(PathBuf::from);
    from_environment
        .into_iter()
        .chain(CA_BUNDLE_CANDIDATES.iter().map(PathBuf::from))
        .find(|candidate| candidate.is_file())
}

fn clear_old_jobs(jobs: &Path) {
    let Ok(entries) = fs::read_dir(jobs) else { return };
    for entry in entries.flatten() {
        let _ = fs::remove_dir_all(entry.path());
    }
}

fn give_to_sandbox_user(path: &Path) -> Result<(), String> {
    let settings = latex_settings();
    std::os::unix::fs::chown(path, Some(settings.sandbox_user), Some(settings.sandbox_group))
        .map_err(|error| io_error("Could not hand the LaTeX cache to the sandbox user", error))
}

fn prepare_jail(root: &Path, binary: &Path, mode: SandboxMode) -> Result<(), String> {
    if needs_dynamic_loader(binary)? {
        return Err("The sandbox needs a statically linked tectonic build, such as the official musl release".to_string());
    }
    for (folder, folder_mode) in [("bin", 0o755), ("etc", 0o755), ("etc/ssl", 0o755), ("etc/ssl/certs", 0o755), ("home", 0o555)] {
        make_directory(&root.join(folder), folder_mode)?;
    }
    install_file(binary, &root.join("bin/tectonic"), 0o755)?;
    let ca_bundle = find_ca_bundle().ok_or("No CA certificate bundle was found, so tectonic could not download LaTeX packages")?;
    install_file(&ca_bundle, &root.join("etc/ssl/certs/ca-certificates.crt"), 0o644)?;
    for network_file in NETWORK_FILES {
        let source = Path::new(network_file);
        if source.is_file() {
            install_file(source, &root.join(network_file.trim_start_matches('/')), 0o644)?;
        }
    }
    let cache = root.join("cache");
    make_directory(&cache, 0o700)?;
    if mode == SandboxMode::Chroot {
        give_to_sandbox_user(&cache)?;
    }
    Ok(())
}

fn prepare_files() -> Result<Sandbox, String> {
    let settings = latex_settings();
    let binary = settings
        .tectonic_path
        .clone()
        .filter(|path| path.is_file())
        .ok_or("tectonic is not installed on the server. Install it or set TECTONIC_PATH")?;
    let mode = choose_mode();

    fs::create_dir_all(&settings.sandbox_root).map_err(|error| io_error("Could not create the LaTeX folder", error))?;
    let root = fs::canonicalize(&settings.sandbox_root).map_err(|error| io_error("Could not find the LaTeX folder", error))?;
    require_private_directory(&root)?;

    if mode == SandboxMode::Unsandboxed {
        tracing::warn!("LATEX_SANDBOX=off: LaTeX files can read any file the server can read. Only use this when every user is trusted.");
        make_directory(&root.join("home"), 0o700)?;
        make_directory(&root.join("cache"), 0o700)?;
    } else {
        prepare_jail(&root, &binary, mode)?;
    }
    let jobs = root.join("jobs");
    make_directory(&jobs, 0o711)?;
    clear_old_jobs(&jobs);

    Ok(Sandbox { root, mode, binary, version: String::new() })
}

async fn check_launch(mut sandbox: Sandbox) -> Result<Sandbox, String> {
    let plan = LaunchPlan { working_directory: "/".to_string(), arguments: vec!["--version".to_string()] };
    let finished = run_tectonic(&sandbox, &plan).await.map_err(|error| format!("tectonic could not start: {error}"))?;
    if !finished.succeeded {
        let detail = finished.output.lines().next().unwrap_or("no output").to_string();
        let hint = match sandbox.mode {
            SandboxMode::Chroot => "the container needs the CHOWN, SETUID, SETGID and SYS_CHROOT capabilities",
            SandboxMode::UserNamespace => "this system does not allow unprivileged user namespaces; run TypstDrive as root in Docker",
            SandboxMode::Unsandboxed => "check that tectonic runs on this system",
        };
        return Err(format!("tectonic did not start in the sandbox ({detail}); {hint}"));
    }
    sandbox.version = finished.output.lines().next().unwrap_or("tectonic").trim().to_string();
    Ok(sandbox)
}

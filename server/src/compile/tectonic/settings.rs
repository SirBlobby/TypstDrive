use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECONDS: u64 = 90;
const DEFAULT_MAX_PARALLEL: usize = 2;
const DEFAULT_MEMORY_MB: u64 = 2048;
const DEFAULT_SANDBOX_ID: u32 = 65534;
const MAX_OUTPUT_FILE_MB: u64 = 256;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SandboxChoice {
    Automatic,
    Off,
}

pub struct LatexSettings {
    pub tectonic_path: Option<PathBuf>,
    pub sandbox_root: PathBuf,
    pub timeout: Duration,
    pub max_parallel: usize,
    pub memory_bytes: u64,
    pub max_file_bytes: u64,
    pub bundle_url: Option<String>,
    pub sandbox: SandboxChoice,
    pub sandbox_user: u32,
    pub sandbox_group: u32,
}

fn number<T: std::str::FromStr>(name: &str, fallback: T) -> T {
    std::env::var(name).ok().and_then(|value| value.trim().parse().ok()).unwrap_or(fallback)
}

fn non_empty(name: &str) -> Option<String> {
    std::env::var(name).ok().map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

fn find_on_path(program: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|directory| directory.join(program))
        .find(|candidate| candidate.is_file())
}

fn sandbox_choice() -> SandboxChoice {
    match non_empty("LATEX_SANDBOX").as_deref() {
        Some("off") => SandboxChoice::Off,
        _ => SandboxChoice::Automatic,
    }
}

pub fn latex_settings() -> &'static LatexSettings {
    static SETTINGS: OnceLock<LatexSettings> = OnceLock::new();
    SETTINGS.get_or_init(|| LatexSettings {
        tectonic_path: non_empty("TECTONIC_PATH").map(PathBuf::from).or_else(|| find_on_path("tectonic")),
        sandbox_root: non_empty("LATEX_DIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("latex-sandbox")),
        timeout: Duration::from_secs(number("LATEX_TIMEOUT_SECONDS", DEFAULT_TIMEOUT_SECONDS).max(5)),
        max_parallel: number("LATEX_MAX_PARALLEL", DEFAULT_MAX_PARALLEL).max(1),
        memory_bytes: number("LATEX_MEMORY_MB", DEFAULT_MEMORY_MB).max(256) * 1024 * 1024,
        max_file_bytes: MAX_OUTPUT_FILE_MB * 1024 * 1024,
        bundle_url: non_empty("LATEX_BUNDLE"),
        sandbox: sandbox_choice(),
        sandbox_user: number("LATEX_SANDBOX_UID", DEFAULT_SANDBOX_ID),
        sandbox_group: number("LATEX_SANDBOX_GID", DEFAULT_SANDBOX_ID),
    })
}

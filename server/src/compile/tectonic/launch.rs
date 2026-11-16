use std::ffi::CString;
use std::io;
use std::process::Stdio;

use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;

use super::sandbox::{Sandbox, SandboxMode, JAILED_BINARY, JAILED_CA_BUNDLE, JAILED_CACHE, JAILED_HOME};
use super::settings::latex_settings;

const MAX_CAPTURED_OUTPUT: usize = 256 * 1024;
const MAX_OPEN_FILES: u64 = 1024;

pub struct LaunchPlan {
    pub working_directory: String,
    pub arguments: Vec<String>,
}

pub struct LaunchResult {
    pub succeeded: bool,
    pub timed_out: bool,
    pub output: String,
}

struct ChildLimits {
    cpu_seconds: u64,
    memory_bytes: u64,
    file_bytes: u64,
}

struct ChildSetup {
    mode: SandboxMode,
    jail_root: CString,
    working_directory: CString,
    sandbox_user: u32,
    sandbox_group: u32,
    user_map: Vec<u8>,
    group_map: Vec<u8>,
    limits: ChildLimits,
}

fn check(result: libc::c_int) -> io::Result<()> {
    if result == -1 { Err(io::Error::last_os_error()) } else { Ok(()) }
}

#[cfg(target_env = "gnu")]
type LimitKind = libc::__rlimit_resource_t;
#[cfg(not(target_env = "gnu"))]
type LimitKind = libc::c_int;

fn set_limit(resource: LimitKind, value: u64) -> io::Result<()> {
    let limit = libc::rlimit { rlim_cur: value as libc::rlim_t, rlim_max: value as libc::rlim_t };
    check(unsafe { libc::setrlimit(resource, &limit) })
}

fn write_proc_file(path: &std::ffi::CStr, contents: &[u8]) -> io::Result<()> {
    let descriptor = unsafe { libc::open(path.as_ptr(), libc::O_WRONLY | libc::O_CLOEXEC) };
    check(descriptor)?;
    let written = unsafe { libc::write(descriptor, contents.as_ptr().cast(), contents.len()) };
    unsafe { libc::close(descriptor) };
    if written != contents.len() as isize {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn enter_jail(setup: &ChildSetup) -> io::Result<()> {
    check(unsafe { libc::chroot(setup.jail_root.as_ptr()) })?;
    check(unsafe { libc::chdir(setup.working_directory.as_ptr()) })
}

fn drop_to_sandbox_user(setup: &ChildSetup) -> io::Result<()> {
    check(unsafe { libc::setgroups(0, std::ptr::null()) })?;
    check(unsafe { libc::setgid(setup.sandbox_group) })?;
    check(unsafe { libc::setuid(setup.sandbox_user) })?;
    if unsafe { libc::setuid(0) } == 0 {
        return Err(io::Error::from(io::ErrorKind::PermissionDenied));
    }
    Ok(())
}

fn isolate_in_user_namespace(setup: &ChildSetup) -> io::Result<()> {
    check(unsafe { libc::unshare(libc::CLONE_NEWUSER) })?;
    write_proc_file(c"/proc/self/setgroups", b"deny")?;
    write_proc_file(c"/proc/self/uid_map", &setup.user_map)?;
    write_proc_file(c"/proc/self/gid_map", &setup.group_map)
}

fn lock_down_child(setup: &ChildSetup) -> io::Result<()> {
    check(unsafe { libc::setsid() })?;
    set_limit(libc::RLIMIT_CORE, 0)?;
    set_limit(libc::RLIMIT_CPU, setup.limits.cpu_seconds)?;
    set_limit(libc::RLIMIT_AS, setup.limits.memory_bytes)?;
    set_limit(libc::RLIMIT_FSIZE, setup.limits.file_bytes)?;
    set_limit(libc::RLIMIT_NOFILE, MAX_OPEN_FILES)?;
    match setup.mode {
        SandboxMode::Chroot => {
            enter_jail(setup)?;
            drop_to_sandbox_user(setup)?;
        }
        SandboxMode::UserNamespace => {
            isolate_in_user_namespace(setup)?;
            enter_jail(setup)?;
        }
        SandboxMode::Unsandboxed => {}
    }
    check(unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) })
}

fn c_string(value: &str) -> io::Result<CString> {
    CString::new(value).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))
}

fn child_setup(sandbox: &Sandbox, plan: &LaunchPlan) -> io::Result<ChildSetup> {
    let settings = latex_settings();
    let user = unsafe { libc::geteuid() };
    let group = unsafe { libc::getegid() };
    Ok(ChildSetup {
        mode: sandbox.mode,
        jail_root: c_string(&sandbox.root.to_string_lossy())?,
        working_directory: c_string(&plan.working_directory)?,
        sandbox_user: settings.sandbox_user,
        sandbox_group: settings.sandbox_group,
        user_map: format!("{user} {user} 1").into_bytes(),
        group_map: format!("{group} {group} 1").into_bytes(),
        limits: ChildLimits {
            cpu_seconds: settings.timeout.as_secs() + 5,
            memory_bytes: settings.memory_bytes,
            file_bytes: settings.max_file_bytes,
        },
    })
}

fn build_command(sandbox: &Sandbox, plan: &LaunchPlan) -> io::Result<Command> {
    let program = if sandbox.is_jailed() { JAILED_BINARY.into() } else { sandbox.binary.clone() };
    let mut command = Command::new(program);
    command
        .args(&plan.arguments)
        .env_clear()
        .env("HOME", sandbox.visible_path(JAILED_HOME))
        .env("TECTONIC_CACHE_DIR", sandbox.visible_path(JAILED_CACHE))
        .env("TECTONIC_UNTRUSTED_MODE", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    if sandbox.is_jailed() {
        command.env("SSL_CERT_FILE", JAILED_CA_BUNDLE);
    } else {
        command.current_dir(&plan.working_directory);
        if let Some(path) = std::env::var_os("PATH") {
            command.env("PATH", path);
        }
    }
    let setup = child_setup(sandbox, plan)?;
    unsafe {
        command.pre_exec(move || lock_down_child(&setup));
    }
    Ok(command)
}

async fn read_limited<R: AsyncRead + Unpin>(reader: Option<R>) -> Vec<u8> {
    let Some(mut reader) = reader else { return Vec::new() };
    let mut kept = Vec::new();
    let mut chunk = [0u8; 8192];
    while let Ok(count) = reader.read(&mut chunk).await {
        if count == 0 {
            break;
        }
        let room = MAX_CAPTURED_OUTPUT.saturating_sub(kept.len());
        kept.extend_from_slice(&chunk[..count.min(room)]);
    }
    kept
}

fn kill_process_group(process_id: Option<u32>) {
    if let Some(process_id) = process_id {
        unsafe { libc::kill(-(process_id as libc::pid_t), libc::SIGKILL) };
    }
}

pub async fn run_tectonic(sandbox: &Sandbox, plan: &LaunchPlan) -> io::Result<LaunchResult> {
    let mut child = build_command(sandbox, plan)?.spawn()?;
    let process_id = child.id();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let finished = {
        let waiting = async { tokio::join!(read_limited(stdout), read_limited(stderr), child.wait()) };
        tokio::time::timeout(latex_settings().timeout, waiting).await
    };
    kill_process_group(process_id);

    let Ok((stdout, stderr, status)) = finished else {
        let _ = child.kill().await;
        return Ok(LaunchResult { succeeded: false, timed_out: true, output: String::new() });
    };
    let mut output = String::from_utf8_lossy(&stdout).into_owned();
    output.push('\n');
    output.push_str(&String::from_utf8_lossy(&stderr));
    Ok(LaunchResult { succeeded: status?.success(), timed_out: false, output })
}

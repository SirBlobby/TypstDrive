use std::sync::Mutex;
use std::time::Instant;

use serde::Serialize;

#[derive(Serialize, Default)]
pub struct SystemView {
    process_memory_bytes: Option<u64>,
    process_threads: Option<u64>,
    process_cpu_percent: Option<f64>,
    cpu_cores: usize,
    load_average: Option<[f64; 3]>,
    memory_total_bytes: Option<u64>,
    memory_available_bytes: Option<u64>,
    disk_path: String,
    disk_total_bytes: Option<u64>,
    disk_free_bytes: Option<u64>,
}

fn status_value(status: &str, field: &str) -> Option<u64> {
    let line = status.lines().find(|line| line.starts_with(field))?;
    line.split_whitespace().nth(1)?.parse().ok()
}

fn process_status() -> (Option<u64>, Option<u64>) {
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return (None, None);
    };
    let memory = status_value(&status, "VmRSS:").map(|kilobytes| kilobytes * 1024);
    (memory, status_value(&status, "Threads:"))
}

fn process_cpu_seconds() -> Option<f64> {
    let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
    let after_name = &stat[stat.rfind(')')? + 2..];
    let fields: Vec<&str> = after_name.split_whitespace().collect();
    let user_ticks: f64 = fields.get(11)?.parse().ok()?;
    let system_ticks: f64 = fields.get(12)?.parse().ok()?;
    let ticks_per_second = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    (ticks_per_second > 0).then(|| (user_ticks + system_ticks) / ticks_per_second as f64)
}

struct CpuSample {
    taken: Instant,
    cpu_seconds: f64,
    percent: Option<f64>,
}

fn process_cpu_percent() -> Option<f64> {
    static LAST_SAMPLE: Mutex<Option<CpuSample>> = Mutex::new(None);
    let cpu_seconds = process_cpu_seconds()?;
    let now = Instant::now();
    let mut last = LAST_SAMPLE.lock().ok()?;
    let (since, cpu_before) = match last.as_ref() {
        Some(sample) if now.duration_since(sample.taken).as_secs_f64() < 1.0 => return sample.percent,
        Some(sample) => (now.duration_since(sample.taken).as_secs_f64(), sample.cpu_seconds),
        None => (super::recorder::uptime().as_secs_f64(), 0.0),
    };
    let percent = (since > 0.0).then(|| ((cpu_seconds - cpu_before) / since * 100.0).max(0.0));
    *last = Some(CpuSample { taken: now, cpu_seconds, percent });
    percent
}

fn load_average() -> Option<[f64; 3]> {
    let text = std::fs::read_to_string("/proc/loadavg").ok()?;
    let mut values = text.split_whitespace().map(|value| value.parse::<f64>().ok());
    Some([values.next()??, values.next()??, values.next()??])
}

fn system_memory() -> (Option<u64>, Option<u64>) {
    let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") else {
        return (None, None);
    };
    let total = status_value(&meminfo, "MemTotal:").map(|kilobytes| kilobytes * 1024);
    let available = status_value(&meminfo, "MemAvailable:").map(|kilobytes| kilobytes * 1024);
    (total, available)
}

fn data_folder() -> std::path::PathBuf {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_default();
    let sqlite_path = database_url.strip_prefix("sqlite://").map(|rest| rest.split('?').next().unwrap_or(rest));
    let parent = sqlite_path
        .map(std::path::Path::new)
        .and_then(|path| path.parent())
        .filter(|parent| !parent.as_os_str().is_empty() && parent.is_dir());
    match parent {
        Some(folder) => folder.to_path_buf(),
        None => std::env::current_dir().unwrap_or_else(|_| ".".into()),
    }
}

fn disk_space(folder: &std::path::Path) -> (Option<u64>, Option<u64>) {
    use std::os::unix::ffi::OsStrExt;
    let Ok(path) = std::ffi::CString::new(folder.as_os_str().as_bytes()) else {
        return (None, None);
    };
    let mut stats: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(path.as_ptr(), &mut stats) } != 0 {
        return (None, None);
    }
    let block_size = stats.f_frsize as u64;
    (Some(stats.f_blocks as u64 * block_size), Some(stats.f_bavail as u64 * block_size))
}

pub fn snapshot() -> SystemView {
    let (process_memory_bytes, process_threads) = process_status();
    let (memory_total_bytes, memory_available_bytes) = system_memory();
    let folder = data_folder();
    let (disk_total_bytes, disk_free_bytes) = disk_space(&folder);
    SystemView {
        process_memory_bytes,
        process_threads,
        process_cpu_percent: process_cpu_percent(),
        cpu_cores: std::thread::available_parallelism().map(|cores| cores.get()).unwrap_or(1),
        load_average: load_average(),
        memory_total_bytes,
        memory_available_bytes,
        disk_path: folder.to_string_lossy().into_owned(),
        disk_total_bytes,
        disk_free_bytes,
    }
}

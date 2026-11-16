use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::timeline::{Timeline, TimelineSummary};

#[derive(Clone, Copy)]
pub enum Metric {
    TypstCompile,
    LatexCompile,
    ApiRequest,
    PageLoad,
}

const ALL_METRICS: [Metric; 4] = [Metric::TypstCompile, Metric::LatexCompile, Metric::ApiRequest, Metric::PageLoad];

impl Metric {
    fn index(self) -> usize {
        self as usize
    }

    pub fn key(self) -> &'static str {
        match self {
            Metric::TypstCompile => "typst_compile",
            Metric::LatexCompile => "latex_compile",
            Metric::ApiRequest => "api_request",
            Metric::PageLoad => "page_load",
        }
    }
}

struct Recorder {
    started: Instant,
    timelines: [Mutex<Timeline>; 4],
}

fn recorder() -> &'static Recorder {
    static RECORDER: OnceLock<Recorder> = OnceLock::new();
    RECORDER.get_or_init(|| Recorder {
        started: Instant::now(),
        timelines: std::array::from_fn(|_| Mutex::new(Timeline::new())),
    })
}

fn current_minute() -> i64 {
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH).map(|elapsed| elapsed.as_secs()).unwrap_or(0);
    (seconds / 60) as i64
}

pub fn start() {
    recorder();
}

pub fn uptime() -> Duration {
    recorder().started.elapsed()
}

pub fn record(metric: Metric, duration: Duration, failed: bool) {
    record_ms(metric, duration.as_secs_f64() * 1000.0, failed);
}

pub fn record_ms(metric: Metric, duration_ms: f64, failed: bool) {
    if !duration_ms.is_finite() || duration_ms < 0.0 {
        return;
    }
    if let Ok(mut timeline) = recorder().timelines[metric.index()].lock() {
        timeline.record(current_minute(), duration_ms, failed);
    }
}

pub async fn timed<T, E>(metric: Metric, work: impl std::future::Future<Output = Result<T, E>>) -> Result<T, E> {
    let started = Instant::now();
    let result = work.await;
    record(metric, started.elapsed(), result.is_err());
    result
}

pub fn summaries(window_minutes: i64, minutes_per_point: i64) -> Vec<(&'static str, TimelineSummary)> {
    let now = current_minute();
    ALL_METRICS
        .iter()
        .filter_map(|metric| {
            let timeline = recorder().timelines[metric.index()].lock().ok()?;
            Some((metric.key(), timeline.summarize(now, window_minutes, minutes_per_point)))
        })
        .collect()
}

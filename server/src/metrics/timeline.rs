use serde::Serialize;

const MINUTES_KEPT: usize = 24 * 60;
const BIN_COUNT: usize = 56;
const BIN_GROWTH: f64 = 1.25;

#[derive(Clone)]
struct Bucket {
    minute: i64,
    count: u64,
    failures: u64,
    total_ms: f64,
    max_ms: f64,
    bins: [u32; BIN_COUNT],
}

impl Bucket {
    fn empty(minute: i64) -> Self {
        Bucket { minute, count: 0, failures: 0, total_ms: 0.0, max_ms: 0.0, bins: [0; BIN_COUNT] }
    }

    fn add(&mut self, other: &Bucket) {
        self.count += other.count;
        self.failures += other.failures;
        self.total_ms += other.total_ms;
        self.max_ms = self.max_ms.max(other.max_ms);
        for (bin, other_bin) in self.bins.iter_mut().zip(other.bins.iter()) {
            *bin += other_bin;
        }
    }

    fn average_ms(&self) -> Option<f64> {
        (self.count > 0).then(|| self.total_ms / self.count as f64)
    }

    fn percentile_ms(&self, share: f64) -> Option<f64> {
        if self.count == 0 {
            return None;
        }
        let wanted = (self.count as f64 * share).ceil().max(1.0) as u64;
        let mut seen = 0u64;
        for (index, amount) in self.bins.iter().enumerate() {
            seen += *amount as u64;
            if seen >= wanted {
                return Some(bin_upper_bound(index).min(self.max_ms));
            }
        }
        Some(self.max_ms)
    }
}

fn bin_index(duration_ms: f64) -> usize {
    if duration_ms <= 1.0 {
        return 0;
    }
    let index = (duration_ms.ln() / BIN_GROWTH.ln()).ceil() as usize;
    index.min(BIN_COUNT - 1)
}

fn bin_upper_bound(index: usize) -> f64 {
    BIN_GROWTH.powi(index as i32)
}

#[derive(Serialize)]
pub struct TimelinePoint {
    time: i64,
    count: u64,
    failures: u64,
    average_ms: Option<f64>,
    p95_ms: Option<f64>,
}

#[derive(Serialize)]
pub struct TimelineSummary {
    count: u64,
    failures: u64,
    average_ms: Option<f64>,
    median_ms: Option<f64>,
    p95_ms: Option<f64>,
    max_ms: Option<f64>,
    points: Vec<TimelinePoint>,
}

pub struct Timeline {
    buckets: Vec<Bucket>,
}

impl Timeline {
    pub fn new() -> Self {
        Timeline { buckets: vec![Bucket::empty(-1); MINUTES_KEPT] }
    }

    pub fn record(&mut self, minute: i64, duration_ms: f64, failed: bool) {
        let slot = minute.rem_euclid(MINUTES_KEPT as i64) as usize;
        let bucket = &mut self.buckets[slot];
        if bucket.minute != minute {
            *bucket = Bucket::empty(minute);
        }
        bucket.count += 1;
        bucket.failures += failed as u64;
        bucket.total_ms += duration_ms;
        bucket.max_ms = bucket.max_ms.max(duration_ms);
        bucket.bins[bin_index(duration_ms)] += 1;
    }

    fn minute(&self, minute: i64) -> Option<&Bucket> {
        let bucket = &self.buckets[minute.rem_euclid(MINUTES_KEPT as i64) as usize];
        (bucket.minute == minute).then_some(bucket)
    }

    pub fn summarize(&self, now_minute: i64, window_minutes: i64, minutes_per_point: i64) -> TimelineSummary {
        let window_minutes = window_minutes.clamp(1, MINUTES_KEPT as i64);
        let minutes_per_point = minutes_per_point.max(1);
        let point_count = (window_minutes + minutes_per_point - 1) / minutes_per_point;
        let first_minute = now_minute - point_count * minutes_per_point + 1;

        let mut total = Bucket::empty(0);
        let mut points = Vec::with_capacity(point_count as usize);
        for point in 0..point_count {
            let start = first_minute + point * minutes_per_point;
            let mut group = Bucket::empty(start);
            for minute in start..start + minutes_per_point {
                if let Some(bucket) = self.minute(minute) {
                    group.add(bucket);
                }
            }
            total.add(&group);
            points.push(TimelinePoint {
                time: start * 60,
                count: group.count,
                failures: group.failures,
                average_ms: group.average_ms(),
                p95_ms: group.percentile_ms(0.95),
            });
        }

        TimelineSummary {
            count: total.count,
            failures: total.failures,
            average_ms: total.average_ms(),
            median_ms: total.percentile_ms(0.5),
            p95_ms: total.percentile_ms(0.95),
            max_ms: (total.count > 0).then_some(total.max_ms),
            points,
        }
    }
}

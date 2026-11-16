use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use serde::{Deserialize, Serialize};

use super::recorder::{record_ms, summaries, uptime, Metric};
use super::system::{snapshot, SystemView};
use super::timeline::TimelineSummary;
use crate::admin::users::require_admin;
use crate::db::sql;
use crate::AppState;

const LONGEST_PAGE_LOAD_MS: f64 = 120_000.0;

#[derive(Deserialize)]
pub struct PeriodQuery {
    period: Option<String>,
}

fn period_minutes(period: Option<&str>) -> (i64, i64) {
    match period {
        Some("6h") => (6 * 60, 5),
        Some("24h") => (24 * 60, 15),
        _ => (60, 1),
    }
}

#[derive(Serialize)]
struct ContentView {
    users: i64,
    documents: i64,
    projects: i64,
    files: i64,
    file_bytes: i64,
}

#[derive(Serialize)]
struct ActivityView {
    live_documents: usize,
    live_connections: usize,
    latex_compiles_running: usize,
    latex_compiles_allowed: usize,
}

#[derive(Serialize)]
pub struct PerformanceView {
    uptime_seconds: u64,
    instance_id: String,
    system: SystemView,
    activity: ActivityView,
    content: ContentView,
    timings: std::collections::BTreeMap<&'static str, TimelineSummary>,
}

async fn count(state: &AppState, query: &'static str) -> i64 {
    sqlx::query_as::<_, (i64,)>(sql(query)).fetch_one(&state.db).await.map(|(value,)| value).unwrap_or(0)
}

async fn content_view(state: &AppState) -> ContentView {
    ContentView {
        users: count(state, "SELECT CAST(COUNT(*) AS BIGINT) FROM users WHERE account_kind = 'user'").await,
        documents: count(state, "SELECT CAST(COUNT(*) AS BIGINT) FROM documents WHERE deleted_at IS NULL").await,
        projects: count(state, "SELECT CAST(COUNT(*) AS BIGINT) FROM projects WHERE deleted_at IS NULL").await,
        files: count(state, "SELECT CAST(COUNT(*) AS BIGINT) FROM files WHERE deleted_at IS NULL").await,
        file_bytes: count(state, "SELECT CAST(COALESCE(SUM(size_bytes), 0) AS BIGINT) FROM files WHERE deleted_at IS NULL").await,
    }
}

async fn activity_view(state: &AppState) -> ActivityView {
    let (live_documents, live_connections) = crate::realtime::rooms::activity(state).await;
    let (latex_compiles_running, latex_compiles_allowed) = crate::compile::tectonic::compile_slots_in_use();
    ActivityView { live_documents, live_connections, latex_compiles_running, latex_compiles_allowed }
}

pub async fn performance(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Query(query): Query<PeriodQuery>,
) -> Result<Json<PerformanceView>, (StatusCode, String)> {
    require_admin(&state, &jar).await?;
    let (window_minutes, minutes_per_point) = period_minutes(query.period.as_deref());
    let system = tokio::task::spawn_blocking(snapshot).await.unwrap_or_default();

    Ok(Json(PerformanceView {
        uptime_seconds: uptime().as_secs(),
        instance_id: state.cluster.instance_id().chars().take(8).collect(),
        system,
        activity: activity_view(&state).await,
        content: content_view(&state).await,
        timings: summaries(window_minutes, minutes_per_point).into_iter().collect(),
    }))
}

#[derive(Deserialize)]
pub struct PageLoadReport {
    duration_ms: f64,
}

pub async fn report_page_load(jar: SignedCookieJar, Json(report): Json<PageLoadReport>) -> StatusCode {
    if jar.get("session_user_id").is_none() {
        return StatusCode::UNAUTHORIZED;
    }
    if report.duration_ms.is_finite() && report.duration_ms > 0.0 && report.duration_ms <= LONGEST_PAGE_LOAD_MS {
        record_ms(Metric::PageLoad, report.duration_ms, false);
    }
    StatusCode::NO_CONTENT
}

use std::time::Instant;

use axum::{extract::Request, middleware::Next, response::Response};

use super::recorder::{record, Metric};

pub async fn time_requests(request: Request, next: Next) -> Response {
    let started = Instant::now();
    let response = next.run(request).await;
    if response.status() != axum::http::StatusCode::SWITCHING_PROTOCOLS {
        record(Metric::ApiRequest, started.elapsed(), response.status().is_server_error());
    }
    response
}

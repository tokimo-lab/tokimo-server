use std::time::{Instant, SystemTime, UNIX_EPOCH};

use axum::{
    extract::{OriginalUri, Request, State},
    middleware::Next,
    response::Response,
};

use crate::{
    metrics::{MetricSample, MetricsAlreadyRecorded},
    AppState,
};

pub async fn record_metrics(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let path = req
        .extensions()
        .get::<OriginalUri>()
        .map(|uri| uri.0.path().to_owned())
        .unwrap_or_else(|| req.uri().path().to_owned());
    let provider = extract_provider(&path);
    let started_at = Instant::now();

    let response = next.run(req).await;

    if response.extensions().get::<MetricsAlreadyRecorded>().is_some() {
        return response;
    }

    if let Some(provider) = provider {
        let duration_ms = started_at.elapsed().as_millis().min(u32::MAX as u128) as u32;
        let status = response.status().as_u16();

        state.metrics.record(MetricSample {
            ts_unix: now_unix(),
            provider,
            status,
            duration_ms,
            cache_hit: false,
        });
    }

    response
}

fn extract_provider(path: &str) -> Option<String> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .find(|segment| !matches!(*segment, "api" | "admin" | "health" | "login"))
        .map(str::to_owned)
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

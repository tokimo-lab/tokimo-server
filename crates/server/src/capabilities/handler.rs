//! `GET /api/capabilities` — public AI-friendly capability discovery.
//!
//! The response is the static [`inventory`] joined with live 24h stats
//! from [`MetricsStore`]. The handler caches the fully-serialized
//! response in memory for `CACHE_TTL` seconds so polling clients don't
//! re-compute percentiles per request.
//!
//! The response itself emits no `Cache-Control` — the `cache_headers`
//! middleware adds the standard `public, max-age=...` + `ETag` for us.

use std::sync::{LazyLock, OnceLock, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

use crate::capabilities::inventory::{self, CategoryInfo, EndpointInfo};
use crate::metrics::ProviderStats;
use crate::AppState;

/// Server-process start time, used to compute uptime in seconds.
/// `get_or_init`-ed by the first call; cost is negligible.
static STARTED_AT: OnceLock<Instant> = OnceLock::new();

/// 30s in-memory response cache (the spec requires this so each poll
/// doesn't re-aggregate p50/p95 for every provider).
const CACHE_TTL: Duration = Duration::from_secs(30);

/// Cache the last successful response body.
static RESPONSE_CACHE: LazyLock<RwLock<Option<CachedResponse>>> = LazyLock::new(|| RwLock::new(None));

struct CachedResponse {
    expires_at: Instant,
    body: serde_json::Value,
}

/// The 24h aggregate stats block at the top of the response.
#[derive(Debug, Clone, Serialize)]
struct GlobalStats24h {
    total_calls: u64,
    total_errors: u64,
    error_rate: f64,
    avg_hit_ratio: f64,
    p50_ms: u32,
    p95_ms: u32,
}

/// Per-provider stats joined into each provider entry.
#[derive(Debug, Clone, Serialize)]
struct ProviderStats24h {
    calls: u64,
    errors: u64,
    p50_ms: u32,
    p95_ms: u32,
    hit_ratio: f64,
    /// One of: "healthy" | "degraded" | "down" | "no-traffic".
    availability: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct ProviderEntry {
    id: &'static str,
    category: &'static str,
    summary: &'static str,
    upstream: &'static str,
    ai_hint: &'static str,
    endpoints: &'static [EndpointInfo],
    #[serde(skip_serializing_if = "Option::is_none")]
    available_ids: Option<&'static [&'static str]>,
    stats_24h: ProviderStats24h,
}

#[derive(Debug, Clone, Serialize)]
struct AuthInfo {
    r#type: &'static str,
    header_format: &'static str,
    public_endpoints: &'static [&'static str],
    admin_endpoints_prefix: &'static str,
    obtain_key: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct CapabilitiesResponse {
    service: &'static str,
    version: &'static str,
    description: &'static str,
    ai_integration_hint: &'static str,
    generated_at: String,
    uptime_seconds: u64,
    stats_window: &'static str,
    auth: AuthInfo,
    global_stats_24h: GlobalStats24h,
    categories: &'static [CategoryInfo],
    providers: Vec<ProviderEntry>,
}

const AI_INTEGRATION_HINT: &str = include_str!("ai_integration_hint.md");

const AUTH: AuthInfo = AuthInfo {
    r#type: "bearer",
    header_format: "Authorization: Bearer <service_key>",
    public_endpoints: &["/api/health", "/api/capabilities"],
    admin_endpoints_prefix: "/api/admin",
    obtain_key: "Contact administrator. Keys start with 'tks_'.",
};

/// Public handler. Returns the cached JSON if still fresh, otherwise
/// rebuilds it from the latest metrics snapshot.
pub async fn capabilities_handler(State(state): State<AppState>) -> impl IntoResponse {
    // First-touch bootstrap: lock in the start instant so uptime works
    // even if the handler is hit before any other code in the binary
    // initialises it. `OnceLock` makes this a one-shot.
    let started = *STARTED_AT.get_or_init(Instant::now);

    if let Some(cached) = read_fresh_cache() {
        return (StatusCode::OK, Json(cached)).into_response();
    }

    let body = build_response(&state, started);
    let json_value = match serde_json::to_value(&body) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!("failed to serialize /api/capabilities: {err}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "serialization error").into_response();
        }
    };

    // Best-effort cache write. We don't fail the request if the lock is
    // poisoned (extremely unlikely, but cheap to handle).
    if let Ok(mut guard) = RESPONSE_CACHE.write() {
        *guard = Some(CachedResponse {
            expires_at: Instant::now() + CACHE_TTL,
            body: json_value.clone(),
        });
    }

    (StatusCode::OK, Json(json_value)).into_response()
}

fn read_fresh_cache() -> Option<serde_json::Value> {
    let guard = RESPONSE_CACHE.read().ok()?;
    let cached = guard.as_ref()?;
    if cached.expires_at > Instant::now() {
        Some(cached.body.clone())
    } else {
        None
    }
}

fn build_response(state: &AppState, started: Instant) -> CapabilitiesResponse {
    // Reuse the same 24h aggregation that backs /api/admin/dashboard/by-provider
    // so this endpoint and the admin dashboard never disagree about what
    // "24h" means.
    let per_provider: Vec<ProviderStats> = state.metrics.query_by_provider(24 * 60 * 60);
    let overview = state.metrics.overview_stats_24h();

    // Global p50/p95 are derived from the timeseries (single 24h bucket)
    // for the same reason — single source of truth.
    let buckets = state.metrics.query_timeseries(24 * 60 * 60, 24 * 60 * 60);
    let (p50_ms, p95_ms) = buckets.first().map(|b| (b.p50_ms, b.p95_ms)).unwrap_or((0, 0));

    let global_stats = GlobalStats24h {
        total_calls: overview.calls_24h,
        total_errors: overview.errors_24h,
        error_rate: safe_ratio(overview.errors_24h, overview.calls_24h),
        avg_hit_ratio: overview.hit_ratio_24h,
        p50_ms,
        p95_ms,
    };

    let providers = inventory::PROVIDERS
        .iter()
        .map(|info| {
            let stats = per_provider.iter().find(|s| s.provider == info.id);
            let stats_24h = match stats {
                Some(s) => ProviderStats24h {
                    calls: s.calls,
                    errors: s.errors,
                    p50_ms: s.p50_ms,
                    p95_ms: s.p95_ms,
                    hit_ratio: s.hit_ratio,
                    availability: classify_availability(s.calls, s.errors),
                },
                None => ProviderStats24h {
                    calls: 0,
                    errors: 0,
                    p50_ms: 0,
                    p95_ms: 0,
                    hit_ratio: 0.0,
                    availability: "no-traffic",
                },
            };

            ProviderEntry {
                id: info.id,
                category: info.category,
                summary: info.summary,
                upstream: info.upstream,
                ai_hint: info.ai_hint,
                endpoints: info.endpoints,
                available_ids: info.available_ids,
                stats_24h,
            }
        })
        .collect::<Vec<_>>();

    CapabilitiesResponse {
        service: "tokimo-server",
        version: env!("CARGO_PKG_VERSION"),
        description: "API proxy + CDN edge for tokimo desktop OS users in network-disadvantaged regions",
        ai_integration_hint: AI_INTEGRATION_HINT,
        generated_at: now_iso8601(),
        uptime_seconds: started.elapsed().as_secs(),
        stats_window: "last 24h",
        auth: AUTH,
        global_stats_24h: global_stats,
        categories: inventory::CATEGORIES,
        providers,
    }
}

/// availability classification rules from the spec:
/// * error_rate > 0.5 AND calls >= 10 → "down"
/// * error_rate > 0.2 AND calls >= 10 → "degraded"
/// * calls > 0                        → "healthy"
/// * calls == 0                       → "no-traffic"
fn classify_availability(calls: u64, errors: u64) -> &'static str {
    if calls == 0 {
        return "no-traffic";
    }
    let error_rate = errors as f64 / calls as f64;
    if calls >= 10 && error_rate > 0.5 {
        "down"
    } else if calls >= 10 && error_rate > 0.2 {
        "degraded"
    } else {
        "healthy"
    }
}

fn safe_ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn now_iso8601() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0)
        .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn availability_rules() {
        assert_eq!(classify_availability(0, 0), "no-traffic");
        assert_eq!(classify_availability(5, 4), "healthy"); // calls<10 short-circuits
        assert_eq!(classify_availability(10, 6), "down");
        assert_eq!(classify_availability(10, 3), "degraded");
        assert_eq!(classify_availability(10, 1), "healthy");
        assert_eq!(classify_availability(100, 1), "healthy");
    }

    #[test]
    fn ai_hint_is_populated() {
        assert!(AI_INTEGRATION_HINT.len() > 200);
        assert!(AI_INTEGRATION_HINT.contains("tokimo-server"));
    }
}

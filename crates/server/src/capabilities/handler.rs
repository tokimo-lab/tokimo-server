//! `GET /api/capabilities` — public AI-friendly capability discovery.
//!
//! Two-tier response:
//!
//! * **No `Authorization` header**: static catalog only. We never touch
//!   the metrics store, so this path is essentially free and we cache
//!   the rendered JSON for 5 minutes.
//! * **Valid `Bearer <service_key>`**: catalog + 24h stats per provider
//!   plus global stats. Aggregation is cached for 30s (the spec) so a
//!   polling client doesn't re-compute p50/p95 every request.
//! * **Invalid key**: falls through to the public view, but with
//!   `auth.warning` set so callers don't mistake it for a full answer.

use std::sync::{LazyLock, OnceLock, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::capabilities::inventory::{self, CategoryInfo, EndpointInfo};
use crate::metrics::ProviderStats;
use crate::middleware::validate_service_key;
use crate::AppState;

static STARTED_AT: OnceLock<Instant> = OnceLock::new();

const AUTHED_CACHE_TTL: Duration = Duration::from_secs(30);
const PUBLIC_CACHE_TTL: Duration = Duration::from_secs(300);

static AUTHED_CACHE: LazyLock<RwLock<Option<CachedResponse>>> = LazyLock::new(|| RwLock::new(None));
static PUBLIC_CACHE: LazyLock<RwLock<Option<CachedResponse>>> = LazyLock::new(|| RwLock::new(None));

struct CachedResponse {
    expires_at: Instant,
    body: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
struct GlobalStats24h {
    total_calls: u64,
    total_errors: u64,
    error_rate: f64,
    avg_hit_ratio: f64,
    p50_ms: u32,
    p95_ms: u32,
}

#[derive(Debug, Clone, Serialize)]
struct ProviderStats24h {
    calls: u64,
    errors: u64,
    p50_ms: u32,
    p95_ms: u32,
    hit_ratio: f64,
    availability: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct ProviderEntryFull {
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
struct ProviderEntryPublic {
    id: &'static str,
    category: &'static str,
    summary: &'static str,
    upstream: &'static str,
    ai_hint: &'static str,
    endpoints: &'static [EndpointInfo],
    #[serde(skip_serializing_if = "Option::is_none")]
    available_ids: Option<&'static [&'static str]>,
}

#[derive(Debug, Clone, Serialize)]
struct AuthInfoPublic {
    r#type: &'static str,
    header_format: &'static str,
    public_endpoints: &'static [&'static str],
    admin_endpoints_prefix: &'static str,
    obtain_key: &'static str,
    current_view: &'static str,
    hint: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    warning: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
struct AuthInfoAuthed {
    r#type: &'static str,
    header_format: &'static str,
    public_endpoints: &'static [&'static str],
    admin_endpoints_prefix: &'static str,
    obtain_key: &'static str,
    current_view: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct CapabilitiesPublic {
    service: &'static str,
    version: &'static str,
    description: &'static str,
    ai_integration_hint: &'static str,
    generated_at: String,
    uptime_seconds: u64,
    stats_window: &'static str,
    auth: AuthInfoPublic,
    categories: &'static [CategoryInfo],
    providers: Vec<ProviderEntryPublic>,
}

#[derive(Debug, Clone, Serialize)]
struct CapabilitiesAuthed {
    service: &'static str,
    version: &'static str,
    description: &'static str,
    ai_integration_hint: &'static str,
    generated_at: String,
    uptime_seconds: u64,
    stats_window: &'static str,
    auth: AuthInfoAuthed,
    global_stats_24h: GlobalStats24h,
    categories: &'static [CategoryInfo],
    providers: Vec<ProviderEntryFull>,
}

const AI_INTEGRATION_HINT: &str = include_str!("ai_integration_hint.md");

const SERVICE: &str = "tokimo-server";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = "API proxy + CDN edge for tokimo desktop OS users in network-disadvantaged regions";
const STATS_WINDOW: &str = "last 24h";

const AUTH_TYPE: &str = "bearer";
const AUTH_HEADER_FORMAT: &str = "Authorization: Bearer <service_key>";
const AUTH_PUBLIC_ENDPOINTS: &[&str] = &["/api/health", "/api/capabilities"];
const AUTH_ADMIN_PREFIX: &str = "/api/admin";
const AUTH_OBTAIN: &str = "Contact administrator. Keys start with 'tks_'.";
const AUTH_PUBLIC_HINT: &str = "Provide 'Authorization: Bearer <service_key>' to see per-provider usage stats.";
const AUTH_INVALID_WARNING: &str = "Provided service key is invalid; showing public view.";

enum AuthState {
    Authorized,
    InvalidKey,
    NoKey,
}

async fn extract_auth_state(state: &AppState, headers: &HeaderMap) -> AuthState {
    let Some(header_value) = headers.get("authorization").and_then(|h| h.to_str().ok()) else {
        return AuthState::NoKey;
    };
    let Some(token) = header_value.strip_prefix("Bearer ") else {
        return AuthState::InvalidKey;
    };
    if validate_service_key(state, token).await.is_some() {
        AuthState::Authorized
    } else {
        AuthState::InvalidKey
    }
}

pub async fn capabilities_handler(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let started = *STARTED_AT.get_or_init(Instant::now);
    let auth_state = extract_auth_state(&state, &headers).await;

    match auth_state {
        AuthState::Authorized => render_authed(&state, started),
        AuthState::InvalidKey => render_public(started, true),
        AuthState::NoKey => render_public(started, false),
    }
}

fn render_authed(state: &AppState, started: Instant) -> axum::response::Response {
    if let Some(cached) = read_fresh(&AUTHED_CACHE) {
        return (StatusCode::OK, Json(cached)).into_response();
    }
    let body = build_authed(state, started);
    let json_value = match serde_json::to_value(&body) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!("failed to serialize authed /api/capabilities: {err}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "serialization error").into_response();
        }
    };
    write_cache(&AUTHED_CACHE, &json_value, AUTHED_CACHE_TTL);
    (StatusCode::OK, Json(json_value)).into_response()
}

fn render_public(started: Instant, invalid_key: bool) -> axum::response::Response {
    // The cached body is identical for NoKey and InvalidKey except for the
    // `auth.warning` field, so cache only the "no warning" variant and
    // inject the warning into a clone when needed.
    let mut json_value = if let Some(cached) = read_fresh(&PUBLIC_CACHE) {
        cached
    } else {
        let body = build_public(started);
        match serde_json::to_value(&body) {
            Ok(v) => {
                write_cache(&PUBLIC_CACHE, &v, PUBLIC_CACHE_TTL);
                v
            }
            Err(err) => {
                tracing::error!("failed to serialize public /api/capabilities: {err}");
                return (StatusCode::INTERNAL_SERVER_ERROR, "serialization error").into_response();
            }
        }
    };

    if invalid_key {
        if let Some(auth) = json_value.get_mut("auth").and_then(|v| v.as_object_mut()) {
            auth.insert(
                "warning".to_string(),
                serde_json::Value::String(AUTH_INVALID_WARNING.to_string()),
            );
        }
    }

    (StatusCode::OK, Json(json_value)).into_response()
}

fn read_fresh(cache: &RwLock<Option<CachedResponse>>) -> Option<serde_json::Value> {
    let guard = cache.read().ok()?;
    let cached = guard.as_ref()?;
    if cached.expires_at > Instant::now() {
        Some(cached.body.clone())
    } else {
        None
    }
}

fn write_cache(cache: &RwLock<Option<CachedResponse>>, value: &serde_json::Value, ttl: Duration) {
    if let Ok(mut guard) = cache.write() {
        *guard = Some(CachedResponse {
            expires_at: Instant::now() + ttl,
            body: value.clone(),
        });
    }
}

fn build_public(started: Instant) -> CapabilitiesPublic {
    let providers = inventory::PROVIDERS
        .iter()
        .map(|info| ProviderEntryPublic {
            id: info.id,
            category: info.category,
            summary: info.summary,
            upstream: info.upstream,
            ai_hint: info.ai_hint,
            endpoints: info.endpoints,
            available_ids: info.available_ids,
        })
        .collect::<Vec<_>>();

    CapabilitiesPublic {
        service: SERVICE,
        version: VERSION,
        description: DESCRIPTION,
        ai_integration_hint: AI_INTEGRATION_HINT,
        generated_at: now_iso8601(),
        uptime_seconds: started.elapsed().as_secs(),
        stats_window: STATS_WINDOW,
        auth: AuthInfoPublic {
            r#type: AUTH_TYPE,
            header_format: AUTH_HEADER_FORMAT,
            public_endpoints: AUTH_PUBLIC_ENDPOINTS,
            admin_endpoints_prefix: AUTH_ADMIN_PREFIX,
            obtain_key: AUTH_OBTAIN,
            current_view: "public",
            hint: AUTH_PUBLIC_HINT,
            warning: None,
        },
        categories: inventory::CATEGORIES,
        providers,
    }
}

fn build_authed(state: &AppState, started: Instant) -> CapabilitiesAuthed {
    let per_provider: Vec<ProviderStats> = state.metrics.query_by_provider(24 * 60 * 60);
    let overview = state.metrics.overview_stats_24h();
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

            ProviderEntryFull {
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

    CapabilitiesAuthed {
        service: SERVICE,
        version: VERSION,
        description: DESCRIPTION,
        ai_integration_hint: AI_INTEGRATION_HINT,
        generated_at: now_iso8601(),
        uptime_seconds: started.elapsed().as_secs(),
        stats_window: STATS_WINDOW,
        auth: AuthInfoAuthed {
            r#type: AUTH_TYPE,
            header_format: AUTH_HEADER_FORMAT,
            public_endpoints: AUTH_PUBLIC_ENDPOINTS,
            admin_endpoints_prefix: AUTH_ADMIN_PREFIX,
            obtain_key: AUTH_OBTAIN,
            current_view: "authorized",
        },
        global_stats_24h: global_stats,
        categories: inventory::CATEGORIES,
        providers,
    }
}

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
        assert_eq!(classify_availability(5, 4), "healthy");
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

    #[test]
    fn ai_hint_mentions_two_tier() {
        assert!(AI_INTEGRATION_HINT.contains("Two-tier"));
    }
}

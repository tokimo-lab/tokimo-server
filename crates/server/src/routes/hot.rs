use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokimo_core::HotItem;
use tokimo_providers::baidu_hot::{create_registry, HotSource};

use crate::metrics::cache_hit;
use crate::{AppError, AppResult, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/sources", get(list_sources))
        .route("/list", get(get_hot_list))
}

#[derive(Serialize)]
struct SourceInfo {
    id: String,
    name: String,
}

async fn list_sources() -> Json<Vec<SourceInfo>> {
    let sources = create_registry();
    let info: Vec<_> = sources
        .iter()
        .map(|s| SourceInfo {
            id: s.id().to_string(),
            name: s.name().to_string(),
        })
        .collect();

    Json(info)
}

#[derive(Deserialize)]
struct HotListQuery {
    id: String,
}

async fn get_hot_list(State(state): State<AppState>, Query(query): Query<HotListQuery>) -> AppResult<Response> {
    let ttl_seconds = state.provider_ttl("hot").await;
    let ttl_duration = Duration::from_secs(ttl_seconds.max(1) as u64);
    let sources = create_registry();
    let source = sources
        .iter()
        .find(|s| s.id() == query.id)
        .ok_or_else(|| AppError::NotFound)?;

    let cache_key = format!("hot:{}", query.id);

    // Check cache first
    if let Some(cached) = state.cache.get("hot", &cache_key).await? {
        if let Ok(items) = serde_json::from_slice::<Vec<HotItem>>(&cached) {
            return Ok(cache_hit(Json(items)));
        }
    }

    // Rate limit
    state.rate_limiter.acquire(&format!("hot_{}", query.id)).await?;

    // Single-flight fetch
    let source_clone: Arc<dyn HotSource> = source.clone();
    let http = state.http.clone();
    let cache = state.cache.clone();
    let cache_key_for_closure = cache_key.clone();

    let items: Vec<HotItem> = state
        .single_flight
        .do_once(&cache_key, move || async move {
            // Race contract: must re-check provider table inside single-flight to
            // handle cross-process losers. For hot lists the "provider table" is
            // the shared PG cache — the first process writes the cache entry
            // before releasing the advisory lock, so losers find it here.
            if let Some(cached) = cache.get("hot", &cache_key_for_closure).await? {
                if let Ok(items) = serde_json::from_slice::<Vec<HotItem>>(&cached) {
                    return Ok(items);
                }
            }

            let span = tracing::info_span!("upstream", provider = "baidu_hot", source = source_clone.id());
            let _enter = span.enter();
            let items = source_clone.fetch(&http).await?;

            // Persist to cache before releasing the advisory lock so cross-process
            // losers observe it on their re-check above.
            if let Ok(serialized) = serde_json::to_vec(&items) {
                let _ = cache
                    .set("hot", &cache_key_for_closure, serialized.into(), ttl_duration)
                    .await;
            }

            Ok(items)
        })
        .await?;

    Ok(Json(items).into_response())
}

/// Background prewarm task: refreshes all 19 hot-search sources at a fixed
/// interval so user requests always observe a warm cache. Bypasses the
/// single-flight/cache-read fast path on purpose — we always perform an
/// upstream fetch and rewrite the cache entry, otherwise a still-valid stale
/// entry would never get refreshed.
///
/// Excluded from `record_metrics` because this path never hits the HTTP layer.
pub async fn prewarm_task(state: AppState) {
    let interval_secs = state.config.prewarm_interval_secs;
    let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
    // Default Burst behavior fires the first tick immediately — desired here.

    loop {
        ticker.tick().await;

        let started = std::time::Instant::now();
        let ttl_seconds = state.provider_ttl("hot").await;
        let ttl_duration = Duration::from_secs(ttl_seconds.max(1) as u64);
        let sources = create_registry();
        let total = sources.len();

        let mut set = tokio::task::JoinSet::new();
        for source in sources {
            let http = state.http.clone();
            let cache = state.cache.clone();
            let ttl = ttl_duration;
            set.spawn(async move {
                let id = source.id();
                let cache_key = format!("hot:{}", id);
                match source.fetch(&http).await {
                    Ok(items) => {
                        if let Ok(serialized) = serde_json::to_vec(&items) {
                            if let Err(e) = cache.set("hot", &cache_key, serialized.into(), ttl).await {
                                tracing::warn!("prewarm cache write failed for source={}, err={:?}", id, e);
                                return false;
                            }
                        }
                        true
                    }
                    Err(e) => {
                        tracing::warn!("prewarm failed for source={}, err={:?}", id, e);
                        false
                    }
                }
            });
        }

        let mut ok = 0usize;
        let mut fail = 0usize;
        while let Some(joined) = set.join_next().await {
            match joined {
                Ok(true) => ok += 1,
                Ok(false) => fail += 1,
                Err(e) => {
                    fail += 1;
                    tracing::warn!("prewarm task panicked: {:?}", e);
                }
            }
        }
        let elapsed_ms = started.elapsed().as_millis();
        tracing::info!(
            "prewarm tick done: {} ok, {} failed (of {}) in {}ms",
            ok,
            fail,
            total,
            elapsed_ms
        );
    }
}

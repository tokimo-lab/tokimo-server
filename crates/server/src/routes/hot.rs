use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokimo_core::HotItem;
use tokimo_providers::baidu_hot::{create_registry, HotSource};

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

async fn get_hot_list(
    State(state): State<AppState>,
    Query(query): Query<HotListQuery>,
) -> AppResult<Json<Vec<HotItem>>> {
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
            return Ok(Json(items));
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

    Ok(Json(items))
}

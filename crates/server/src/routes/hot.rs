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

    let items: Vec<HotItem> = state
        .single_flight
        .do_once(&cache_key, || async move {
            let span = tracing::info_span!("upstream", provider = "baidu_hot", source = source_clone.id());
            let _enter = span.enter();
            source_clone.fetch(&http).await
        })
        .await?;

    // Cache for 2 minutes
    let serialized =
        serde_json::to_vec(&items).map_err(|e| AppError::Internal(format!("Serialization failed: {}", e)))?;
    state
        .cache
        .set("hot", &cache_key, serialized.into(), Duration::from_secs(120))
        .await?;

    Ok(Json(items))
}

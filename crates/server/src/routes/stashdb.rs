use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use tokimo_providers::stashdb;

use crate::{AppError, AppResult, AppState};

pub fn routes() -> Router<AppState> {
    Router::new().route("/search", get(search))
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    video_id: String,
}

fn require_config(state: &AppState) -> AppResult<(String, Option<String>)> {
    let base_url = state
        .config
        .stashdb_base_url
        .clone()
        .ok_or_else(|| AppError::Internal("STASHDB_BASE_URL not configured".to_string()))?;
    Ok((base_url, state.config.stashdb_api_key.clone()))
}

async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<Option<serde_json::Value>>> {
    let video_id = query.video_id.trim().to_string();
    if video_id.is_empty() {
        return Err(AppError::BadRequest("video_id must not be empty".to_string()));
    }

    let (base_url, api_key) = require_config(&state)?;

    state.rate_limiter.acquire("stashdb").await?;

    let cache_key = format!("stashdb:search:{video_id}");
    let http = state.http.clone();
    let video_id_clone = video_id.clone();

    let result = state
        .single_flight
        .do_once(&cache_key, move || async move {
            stashdb::search_by_video_id(&http, &base_url, &video_id_clone, api_key.as_deref()).await
        })
        .await?;

    Ok(Json(result))
}

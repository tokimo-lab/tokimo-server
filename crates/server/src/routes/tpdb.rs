use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use tokimo_providers::tpdb;

use crate::{AppError, AppResult, AppState};

pub fn routes() -> Router<AppState> {
    Router::new().route("/search", get(search))
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    video_id: String,
}

fn require_config(state: &AppState) -> AppResult<(String, String)> {
    let api_key = state
        .config
        .tpdb_api_key
        .clone()
        .ok_or_else(|| AppError::Internal("TPDB_API_KEY not configured".to_string()))?;
    let base_url = state
        .config
        .tpdb_base_url
        .clone()
        .ok_or_else(|| AppError::Internal("TPDB_BASE_URL not configured".to_string()))?;
    Ok((api_key, base_url))
}

async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<Option<serde_json::Value>>> {
    let video_id = query.video_id.trim().to_string();
    if video_id.is_empty() {
        return Err(AppError::BadRequest("video_id must not be empty".to_string()));
    }

    let (api_key, base_url) = require_config(&state)?;

    state.rate_limiter.acquire("tpdb").await?;

    let cache_key = format!("tpdb:search:{video_id}");
    let http = state.http.clone();
    let video_id_clone = video_id.clone();

    let result = state
        .single_flight
        .do_once(&cache_key, move || async move {
            tpdb::search_by_video_id(&http, &api_key, &base_url, &video_id_clone).await
        })
        .await?;

    Ok(Json(result))
}

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_providers::assrt;

use crate::{
    db::entities::{assrt_searches, assrt_sub_details, AssrtSearches, AssrtSubDetails},
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/search", get(get_search))
        .route("/sub/:id/detail", get(get_detail))
}

fn require_token(state: &AppState) -> AppResult<String> {
    state
        .config
        .assrt_api_key
        .clone()
        .ok_or_else(|| AppError::Internal("ASSRT_API_KEY not configured".into()))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub cnt: Option<u32>,
    pub pos: Option<u32>,
}

async fn get_search(State(state): State<AppState>, Query(q): Query<SearchQuery>) -> AppResult<Response> {
    let key = assrt::search_cache_key(&q.q, q.cnt, q.pos);

    if let Some(row) = AssrtSearches::find_by_id(key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(cache_hit(Json(row.raw_json)));
    }

    let token = require_token(&state)?;
    state.rate_limiter.acquire("assrt").await?;

    let cache_key_sf = format!("assrt:search:{}", key);
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();
    let q_owned = q.q.clone();
    let cnt = q.cnt;
    let pos = q.pos;

    let raw_json = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = AssrtSearches::find_by_id(key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = assrt::search(&http, &token, &q_owned, cnt, pos).await?;

            let am = assrt_searches::ActiveModel {
                cache_key: Set(key_clone.clone()),
                query: Set(q_owned.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            AssrtSearches::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json).into_response())
}

async fn get_detail(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Response> {
    if let Some(row) = AssrtSubDetails::find_by_id(id.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(cache_hit(Json(row.raw_json)));
    }

    let token = require_token(&state)?;
    state.rate_limiter.acquire("assrt").await?;

    let cache_key_sf = format!("assrt:detail:{}", id);
    let http = state.http.clone();
    let db = state.db.clone();
    let id_clone = id.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = AssrtSubDetails::find_by_id(id_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = assrt::fetch_detail(&http, &token, &id_clone).await?;

            let am = assrt_sub_details::ActiveModel {
                sub_id: Set(id_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            AssrtSubDetails::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json).into_response())
}

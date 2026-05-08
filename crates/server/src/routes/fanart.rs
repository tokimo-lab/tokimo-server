use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use tokimo_providers::fanart;

use crate::{
    db::entities::{fanart_assets, FanartAssets},
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/movie/:tmdb_id", get(get_movie))
        .route("/tv/:tvdb_id", get(get_tv))
}

fn require_key(state: &AppState) -> AppResult<String> {
    state
        .config
        .fanart_api_key
        .clone()
        .ok_or_else(|| AppError::Internal("FANART_API_KEY not configured".into()))
}

async fn fetch_or_cache(state: AppState, kind: &'static str, foreign_id: i64) -> AppResult<Response> {
    if let Some(row) = FanartAssets::find_by_id((kind.to_string(), foreign_id))
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(cache_hit(Json(row.raw_json)));
    }

    let api_key = require_key(&state)?;
    state.rate_limiter.acquire("fanart").await?;

    let cache_key = format!("fanart:{}:{}", kind, foreign_id);
    let http = state.http.clone();
    let db = state.db.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = FanartAssets::find_by_id((kind.to_string(), foreign_id))
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let opt = match kind {
                "movie" => fanart::fetch_movie_images(&http, &api_key, None, foreign_id).await?,
                "tv" => fanart::fetch_tv_images(&http, &api_key, None, foreign_id).await?,
                _ => unreachable!(),
            };
            let raw = match opt {
                Some(v) => v,
                None => return Err(tokimo_core::CoreError::NotFound),
            };

            let am = fanart_assets::ActiveModel {
                kind: Set(kind.to_string()),
                foreign_id: Set(foreign_id),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            FanartAssets::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json).into_response())
}

async fn get_movie(State(state): State<AppState>, Path(tmdb_id): Path<i64>) -> AppResult<Response> {
    fetch_or_cache(state, "movie", tmdb_id).await
}

async fn get_tv(State(state): State<AppState>, Path(tvdb_id): Path<i64>) -> AppResult<Response> {
    fetch_or_cache(state, "tv", tvdb_id).await
}

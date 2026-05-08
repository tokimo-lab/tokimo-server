use std::time::Instant;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_providers::lrclib;

use crate::{
    db::entities::{lrclib_lyrics, LrclibLyrics},
    metrics::cache_hit_response,
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/get", get(get_lyrics))
        .route("/search", get(get_search))
}

#[derive(Deserialize)]
pub struct GetQuery {
    pub artist: String,
    pub track: String,
    pub album: Option<String>,
    pub duration: Option<u32>,
}

async fn get_lyrics(State(state): State<AppState>, Query(q): Query<GetQuery>) -> AppResult<Response> {
    let started = Instant::now();
    let key = lrclib::cache_key(&q.artist, &q.track, q.album.as_deref(), q.duration);

    if let Some(row) = LrclibLyrics::find_by_id(key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(cache_hit_response(&state, "lrclib", started, Json(row.raw_json)));
    }

    state.rate_limiter.acquire("lrclib").await?;

    let cache_key_sf = format!("lrclib:get:{}", key);
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();
    let artist = q.artist.clone();
    let track = q.track.clone();
    let album = q.album.clone();
    let duration = q.duration;

    let raw_json = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = LrclibLyrics::find_by_id(key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = lrclib::fetch_lyrics(&http, &artist, &track, album.as_deref(), duration).await?;

            let am = lrclib_lyrics::ActiveModel {
                cache_key: Set(key_clone.clone()),
                artist: Set(artist.clone()),
                track: Set(track.clone()),
                album: Set(album.clone()),
                duration: Set(duration.map(|d| d as i32)),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            LrclibLyrics::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json).into_response())
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

/// Search results are not persisted; only rate limiter + single-flight.
async fn get_search(State(state): State<AppState>, Query(q): Query<SearchQuery>) -> AppResult<Json<serde_json::Value>> {
    state.rate_limiter.acquire("lrclib").await?;

    let cache_key_sf = format!("lrclib:search:{}", q.q);
    let http = state.http.clone();
    let q_owned = q.q.clone();

    let raw = state
        .single_flight
        .do_once(
            &cache_key_sf,
            move || async move { lrclib::search(&http, &q_owned).await },
        )
        .await?;

    Ok(Json(raw))
}

use std::time::Instant;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Serialize;
use tokimo_providers::{
    common::download_to_storage,
    tmdb::{download_image, fetch_movie, fetch_person, fetch_tv, fetch_tv_episode, fetch_tv_season},
};

use crate::{
    db::entities::{tmdb_images, tmdb_movies, tmdb_objects, TmdbImages, TmdbMovies, TmdbObjects},
    metrics::cache_hit_response,
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/movie/:id", get(get_movie))
        .route("/tv/:id", get(get_tv))
        .route("/tv/:id/season/:n", get(get_tv_season))
        .route("/tv/:id/season/:s/episode/:e", get(get_tv_episode))
        .route("/person/:id", get(get_person))
        .route("/image/*path", get(get_image))
}

#[derive(Serialize)]
struct MovieResponse {
    id: i32,
    title: String,
    original_title: Option<String>,
    overview: Option<String>,
    release_date: Option<String>,
    runtime: Option<i32>,
    vote_average: Option<f64>,
    vote_count: Option<i32>,
    poster_url: Option<String>,
    backdrop_url: Option<String>,
}

async fn get_movie(State(state): State<AppState>, Path(id): Path<i32>) -> AppResult<Response> {
    let started = Instant::now();
    // Fast-path DB check before single-flight: avoids both a local lock and a
    // PG round-trip in the common cache-hit case.
    if let Some(movie) = TmdbMovies::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        let resp = movie_to_response(&state, movie).await?;
        return Ok(cache_hit_response(&state, "tmdb", started, Json(resp)));
    }

    // DB miss - fetch from upstream with single-flight
    let api_key = state
        .config
        .tmdb_api_key
        .as_ref()
        .ok_or_else(|| AppError::Internal("TMDB API key not configured".into()))?;

    state.rate_limiter.acquire("tmdb").await?;

    let cache_key = format!("movie:{}", id);
    let http = state.http.clone();
    let storage = state.storage.clone();
    let api_key_clone = api_key.clone();
    let db = state.db.clone();

    let movie: tmdb_movies::Model = state
        .single_flight
        .do_once(&cache_key, move || async move {
            // Race contract: must re-check provider table inside single-flight to
            // handle cross-process losers. The first process to acquire the PG
            // advisory lock writes the row; everyone else wakes up here, finds
            // it, and short-circuits without re-hitting the upstream.
            if let Some(movie) = TmdbMovies::find_by_id(id)
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(movie);
            }

            let span = tracing::info_span!("upstream", provider = "tmdb", movie_id = id);
            let _enter = span.enter();

            let (movie, raw_json) = fetch_movie(&http, &api_key_clone, id).await?;

            let poster_key = if let Some(poster_path) = &movie.poster_path {
                let (_, key) = download_image(&http, poster_path, storage.as_ref()).await?;
                Some(key)
            } else {
                None
            };

            let backdrop_key = if let Some(backdrop_path) = &movie.backdrop_path {
                let (_, key) = download_image(&http, backdrop_path, storage.as_ref()).await?;
                Some(key)
            } else {
                None
            };

            let model = tmdb_movies::ActiveModel {
                id: Set(id),
                title: Set(raw_json["title"].as_str().unwrap_or("").to_string()),
                original_title: Set(raw_json["original_title"].as_str().map(|s| s.to_string())),
                overview: Set(raw_json["overview"].as_str().map(|s| s.to_string())),
                release_date: Set(raw_json["release_date"].as_str().map(|s| s.to_string())),
                runtime: Set(raw_json["runtime"].as_i64().map(|n| n as i32)),
                vote_average: Set(raw_json["vote_average"].as_f64()),
                vote_count: Set(raw_json["vote_count"].as_i64().map(|n| n as i32)),
                poster_storage_key: Set(poster_key),
                backdrop_storage_key: Set(backdrop_key),
                raw_json: Set(raw_json),
                fetched_at: Set(chrono::Utc::now().into()),
            };

            let inserted = TmdbMovies::insert(model.clone())
                .exec_with_returning(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(inserted)
        })
        .await?;

    let resp = movie_to_response(&state, movie).await?;
    Ok(Json(resp).into_response())
}

async fn movie_to_response(state: &AppState, movie: tmdb_movies::Model) -> AppResult<MovieResponse> {
    let poster_url = match &movie.poster_storage_key {
        Some(k) => Some(
            state
                .storage
                .url_for(k)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?,
        ),
        None => None,
    };
    let backdrop_url = match &movie.backdrop_storage_key {
        Some(k) => Some(
            state
                .storage
                .url_for(k)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?,
        ),
        None => None,
    };
    Ok(MovieResponse {
        id: movie.id,
        title: movie.title,
        original_title: movie.original_title,
        overview: movie.overview,
        release_date: movie.release_date,
        runtime: movie.runtime,
        vote_average: movie.vote_average,
        vote_count: movie.vote_count,
        poster_url,
        backdrop_url,
    })
}

// ─── generic JSON object endpoints (tv / season / episode / person) ────────

async fn fetch_or_cache_object(
    state: &AppState,
    started: Instant,
    kind: &'static str,
    key: String,
    upstream: impl FnOnce(reqwest::Client, String) -> futures_util_compat::BoxFut + Send + 'static,
) -> AppResult<Response> {
    if let Some(obj) = TmdbObjects::find_by_id((kind.to_string(), key.clone()))
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(cache_hit_response(state, "tmdb", started, Json(obj.raw_json)));
    }

    let api_key = state
        .config
        .tmdb_api_key
        .as_ref()
        .ok_or_else(|| AppError::Internal("TMDB API key not configured".into()))?
        .clone();

    state.rate_limiter.acquire("tmdb").await?;

    let cache_key = format!("tmdb:{}:{}", kind, key);
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();

    let raw_json: serde_json::Value = state
        .single_flight
        .do_once(&cache_key, move || async move {
            // Re-check inside single-flight (cross-process winner may have written it).
            if let Some(obj) = TmdbObjects::find_by_id((kind.to_string(), key_clone.clone()))
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(obj.raw_json);
            }

            let raw_json = upstream(http, api_key).await?;

            let am = tmdb_objects::ActiveModel {
                kind: Set(kind.to_string()),
                key: Set(key_clone.clone()),
                raw_json: Set(raw_json.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            TmdbObjects::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw_json)
        })
        .await?;

    Ok(Json(raw_json).into_response())
}

mod futures_util_compat {
    use std::future::Future;
    use std::pin::Pin;
    pub type BoxFut = Pin<Box<dyn Future<Output = tokimo_core::CoreResult<serde_json::Value>> + Send>>;
}

async fn get_tv(State(state): State<AppState>, Path(id): Path<i32>) -> AppResult<Response> {
    let started = Instant::now();
    fetch_or_cache_object(&state, started, "tv", id.to_string(), move |http, api_key| {
        Box::pin(async move { tokimo_providers::tmdb::fetch_tv(&http, &api_key, id).await })
    })
    .await
}

async fn get_tv_season(State(state): State<AppState>, Path((id, n)): Path<(i32, i32)>) -> AppResult<Response> {
    let started = Instant::now();
    fetch_or_cache_object(
        &state,
        started,
        "tv_season",
        format!("{}:{}", id, n),
        move |http, api_key| Box::pin(async move { fetch_tv_season(&http, &api_key, id, n).await }),
    )
    .await
}

async fn get_tv_episode(State(state): State<AppState>, Path((id, s, e)): Path<(i32, i32, i32)>) -> AppResult<Response> {
    let started = Instant::now();
    fetch_or_cache_object(
        &state,
        started,
        "tv_episode",
        format!("{}:{}:{}", id, s, e),
        move |http, api_key| Box::pin(async move { fetch_tv_episode(&http, &api_key, id, s, e).await }),
    )
    .await
}

async fn get_person(State(state): State<AppState>, Path(id): Path<i32>) -> AppResult<Response> {
    let started = Instant::now();
    fetch_or_cache_object(&state, started, "person", id.to_string(), move |http, api_key| {
        Box::pin(async move { fetch_person(&http, &api_key, id).await })
    })
    .await
}

// ─── image proxy ────────────────────────────────────────────────────────────

/// `/api/tmdb/image/*path` — accepts any TMDB image path (e.g.
/// `original/abc.jpg` or `w500/def.png`); caches once to storage and
/// redirects subsequent hits to the storage public URL.
async fn get_image(State(state): State<AppState>, Path(path): Path<String>) -> AppResult<Response> {
    let started = Instant::now();
    // Normalize: caller may pass with or without leading slash; we always
    // store with a leading slash so cache keys are stable.
    let normalized = if path.starts_with('/') {
        path.clone()
    } else {
        format!("/{}", path)
    };

    if let Some(row) = TmdbImages::find_by_id(normalized.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        let url = state
            .storage
            .url_for(&row.storage_key)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        return Ok(cache_hit_response(&state, "tmdb", started, Redirect::temporary(&url)));
    }

    state.rate_limiter.acquire("tmdb").await?;

    let cache_key = format!("tmdb:image:{}", normalized);
    let http = state.http.clone();
    let storage = state.storage.clone();
    let db = state.db.clone();
    let path_clone = normalized.clone();

    let stored: tmdb_images::Model = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = TmdbImages::find_by_id(path_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row);
            }

            // Use the path as-is against the TMDB CDN. `download_image` already
            // prepends https://image.tmdb.org/t/p/original — but the API also
            // supports per-size paths (w500/foo.jpg). We support both: if the
            // path contains a size segment (first segment is a size token),
            // hit the CDN root; otherwise treat as /original/.
            let cdn_url = if path_clone
                .trim_start_matches('/')
                .split_once('/')
                .map(|(seg, _)| seg.starts_with('w') || seg == "original")
                .unwrap_or(false)
            {
                format!("https://image.tmdb.org/t/p{}", path_clone)
            } else {
                format!("https://image.tmdb.org/t/p/original{}", path_clone)
            };

            let (sha, key) = download_to_storage(&http, &cdn_url, storage.as_ref(), "tmdb").await?;

            let am = tmdb_images::ActiveModel {
                image_path: Set(path_clone.clone()),
                storage_key: Set(key),
                sha256: Set(sha),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            let inserted = TmdbImages::insert(am)
                .exec_with_returning(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(inserted)
        })
        .await?;

    let url = state
        .storage
        .url_for(&stored.storage_key)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Redirect::temporary(&url).into_response())
}

// keep existing helpers used above
#[allow(dead_code)]
fn _suppress_unused() {
    // tells clippy that fetch_tv is intentionally re-imported via use { ... }
    let _ = fetch_tv;
    let _ = fetch_movie;
    let _ = download_image;
}

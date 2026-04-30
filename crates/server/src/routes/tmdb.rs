use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Serialize;
use tokimo_providers::tmdb::{download_image, fetch_movie};

use crate::{
    db::entities::{tmdb_movies, TmdbMovies},
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/movie/:id", get(get_movie))
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

async fn get_movie(State(state): State<AppState>, Path(id): Path<i32>) -> AppResult<Json<MovieResponse>> {
    // Check DB first
    if let Some(movie) = TmdbMovies::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(Json(MovieResponse {
            id: movie.id,
            title: movie.title,
            original_title: movie.original_title,
            overview: movie.overview,
            release_date: movie.release_date,
            runtime: movie.runtime,
            vote_average: movie.vote_average,
            vote_count: movie.vote_count,
            poster_url: movie.poster_storage_key.map(|k| state.storage.url_for(&k)),
            backdrop_url: movie.backdrop_storage_key.map(|k| state.storage.url_for(&k)),
        }));
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

    let result: (serde_json::Value, Option<String>, Option<String>) = state
        .single_flight
        .do_once(&cache_key, move || async move {
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

            Ok((raw_json, poster_key, backdrop_key))
        })
        .await?;

    let (raw_json, poster_key, backdrop_key) = result;

    // Parse and persist
    let movie_data: serde_json::Value = raw_json;

    let model = tmdb_movies::ActiveModel {
        id: Set(id),
        title: Set(movie_data["title"].as_str().unwrap_or("").to_string()),
        original_title: Set(movie_data["original_title"].as_str().map(|s| s.to_string())),
        overview: Set(movie_data["overview"].as_str().map(|s| s.to_string())),
        release_date: Set(movie_data["release_date"].as_str().map(|s| s.to_string())),
        runtime: Set(movie_data["runtime"].as_i64().map(|n| n as i32)),
        vote_average: Set(movie_data["vote_average"].as_f64()),
        vote_count: Set(movie_data["vote_count"].as_i64().map(|n| n as i32)),
        poster_storage_key: Set(poster_key.clone()),
        backdrop_storage_key: Set(backdrop_key.clone()),
        raw_json: Set(movie_data.clone()),
        fetched_at: Set(chrono::Utc::now().into()),
    };

    TmdbMovies::insert(model)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(MovieResponse {
        id,
        title: movie_data["title"].as_str().unwrap_or("").to_string(),
        original_title: movie_data["original_title"].as_str().map(|s| s.to_string()),
        overview: movie_data["overview"].as_str().map(|s| s.to_string()),
        release_date: movie_data["release_date"].as_str().map(|s| s.to_string()),
        runtime: movie_data["runtime"].as_i64().map(|n| n as i32),
        vote_average: movie_data["vote_average"].as_f64(),
        vote_count: movie_data["vote_count"].as_i64().map(|n| n as i32),
        poster_url: poster_key.map(|k| state.storage.url_for(&k)),
        backdrop_url: backdrop_key.map(|k| state.storage.url_for(&k)),
    }))
}

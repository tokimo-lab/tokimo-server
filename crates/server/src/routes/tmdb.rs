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
    // Fast-path DB check before single-flight: avoids both a local lock and a
    // PG round-trip in the common cache-hit case.
    if let Some(movie) = TmdbMovies::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return movie_to_response(&state, movie).await.map(Json);
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

    movie_to_response(&state, movie).await.map(Json)
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

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_providers::deezer;

use crate::{
    db::entities::{deezer_albums, deezer_artists, deezer_tracks, DeezerAlbums, DeezerArtists, DeezerTracks},
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/track/:id", get(get_track))
        .route("/album/:id", get(get_album))
        .route("/artist/:id", get(get_artist))
        .route("/search", get(get_search))
}

async fn get_track(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Response> {
    if let Some(row) = DeezerTracks::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(cache_hit(Json(row.raw_json)));
    }

    state.rate_limiter.acquire("deezer").await?;

    let cache_key = format!("deezer:track:{}", id);
    let http = state.http.clone();
    let db = state.db.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = DeezerTracks::find_by_id(id)
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = deezer::fetch_track(&http, id).await?;

            let am = deezer_tracks::ActiveModel {
                deezer_id: Set(id),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            DeezerTracks::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json).into_response())
}

async fn get_album(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Response> {
    if let Some(row) = DeezerAlbums::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(cache_hit(Json(row.raw_json)));
    }

    state.rate_limiter.acquire("deezer").await?;

    let cache_key = format!("deezer:album:{}", id);
    let http = state.http.clone();
    let db = state.db.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = DeezerAlbums::find_by_id(id)
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = deezer::fetch_album(&http, id).await?;

            let am = deezer_albums::ActiveModel {
                deezer_id: Set(id),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            DeezerAlbums::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json).into_response())
}

async fn get_artist(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Response> {
    if let Some(row) = DeezerArtists::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(cache_hit(Json(row.raw_json)));
    }

    state.rate_limiter.acquire("deezer").await?;

    let cache_key = format!("deezer:artist:{}", id);
    let http = state.http.clone();
    let db = state.db.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = DeezerArtists::find_by_id(id)
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = deezer::fetch_artist(&http, id).await?;

            let am = deezer_artists::ActiveModel {
                deezer_id: Set(id),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            DeezerArtists::insert(am)
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

/// Search is not persisted (high cardinality query strings); only the
/// rate limiter + single-flight coalesce identical concurrent queries.
async fn get_search(State(state): State<AppState>, Query(q): Query<SearchQuery>) -> AppResult<Json<serde_json::Value>> {
    state.rate_limiter.acquire("deezer").await?;

    let cache_key = format!("deezer:search:{}", q.q);
    let http = state.http.clone();
    let q_owned = q.q.clone();

    let raw = state
        .single_flight
        .do_once(&cache_key, move || async move { deezer::search(&http, &q_owned).await })
        .await?;

    Ok(Json(raw))
}

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_providers::spotify;

use crate::{
    db::entities::{
        spotify_albums, spotify_artists, spotify_token_cache, spotify_tracks, SpotifyAlbums, SpotifyArtists,
        SpotifyTokenCache, SpotifyTracks,
    },
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/artist/:id", get(get_artist))
        .route("/album/:id", get(get_album))
        .route("/track/:id", get(get_track))
        .route("/search", get(get_search))
}

const TOKEN_ROW_ID: i32 = 1;
/// Refresh tokens 60s before stated expiry to avoid edge-case clock skew.
const TOKEN_SAFETY_MARGIN_SECONDS: i64 = 60;

fn require_credentials(state: &AppState) -> AppResult<(String, String)> {
    let id = state
        .config
        .spotify_client_id
        .clone()
        .ok_or_else(|| AppError::Internal("SPOTIFY_CLIENT_ID not configured".into()))?;
    let secret = state
        .config
        .spotify_client_secret
        .clone()
        .ok_or_else(|| AppError::Internal("SPOTIFY_CLIENT_SECRET not configured".into()))?;
    Ok((id, secret))
}

/// Get a valid bearer token, refreshing via /api/token if missing/expired.
async fn ensure_token(state: &AppState) -> AppResult<String> {
    let now = chrono::Utc::now();

    let row = SpotifyTokenCache::find_by_id(TOKEN_ROW_ID)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(r) = row {
        let expires = r.expires_at.naive_utc().and_utc();
        if expires - chrono::Duration::seconds(TOKEN_SAFETY_MARGIN_SECONDS) > now {
            return Ok(r.access_token);
        }
    }

    let (client_id, client_secret) = require_credentials(state)?;
    let http = state.http.clone();
    let db = state.db.clone();

    let token = state
        .single_flight
        .do_once::<String, _, _>("spotify:token", move || async move {
            // Re-check inside single-flight; another node may have refreshed.
            let row = SpotifyTokenCache::find_by_id(TOKEN_ROW_ID)
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;
            let now = chrono::Utc::now();
            if let Some(r) = row {
                if r.expires_at.naive_utc().and_utc() - chrono::Duration::seconds(TOKEN_SAFETY_MARGIN_SECONDS) > now {
                    return Ok(r.access_token);
                }
            }

            let issued = spotify::request_token(&http, &client_id, &client_secret).await?;
            let expires_at = now + chrono::Duration::seconds(issued.expires_in);

            let am = spotify_token_cache::ActiveModel {
                id: Set(TOKEN_ROW_ID),
                access_token: Set(issued.access_token.clone()),
                expires_at: Set(expires_at.into()),
            };
            SpotifyTokenCache::insert(am)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(spotify_token_cache::Column::Id)
                        .update_columns([
                            spotify_token_cache::Column::AccessToken,
                            spotify_token_cache::Column::ExpiresAt,
                        ])
                        .to_owned(),
                )
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(issued.access_token)
        })
        .await?;

    Ok(token)
}

async fn get_artist(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Response> {
    if let Some(row) = SpotifyArtists::find_by_id(id.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(cache_hit(Json(row.raw_json)));
    }

    state.rate_limiter.acquire("spotify").await?;
    let token = ensure_token(&state).await?;

    let cache_key = format!("spotify:artist:{}", id);
    let http = state.http.clone();
    let db = state.db.clone();
    let id_clone = id.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = SpotifyArtists::find_by_id(id_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = spotify::fetch_artist(&http, &token, &id_clone).await?;

            let am = spotify_artists::ActiveModel {
                spotify_id: Set(id_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            SpotifyArtists::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json).into_response())
}

async fn get_album(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Response> {
    if let Some(row) = SpotifyAlbums::find_by_id(id.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(cache_hit(Json(row.raw_json)));
    }

    state.rate_limiter.acquire("spotify").await?;
    let token = ensure_token(&state).await?;

    let cache_key = format!("spotify:album:{}", id);
    let http = state.http.clone();
    let db = state.db.clone();
    let id_clone = id.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = SpotifyAlbums::find_by_id(id_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = spotify::fetch_album(&http, &token, &id_clone).await?;

            let am = spotify_albums::ActiveModel {
                spotify_id: Set(id_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            SpotifyAlbums::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json).into_response())
}

async fn get_track(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Response> {
    if let Some(row) = SpotifyTracks::find_by_id(id.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(cache_hit(Json(row.raw_json)));
    }

    state.rate_limiter.acquire("spotify").await?;
    let token = ensure_token(&state).await?;

    let cache_key = format!("spotify:track:{}", id);
    let http = state.http.clone();
    let db = state.db.clone();
    let id_clone = id.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = SpotifyTracks::find_by_id(id_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = spotify::fetch_track(&http, &token, &id_clone).await?;

            let am = spotify_tracks::ActiveModel {
                spotify_id: Set(id_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            SpotifyTracks::insert(am)
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
    #[serde(rename = "type")]
    pub type_: String,
}

/// Search is not persisted (high cardinality); we only apply the rate
/// limiter + single-flight to coalesce identical concurrent queries.
async fn get_search(State(state): State<AppState>, Query(q): Query<SearchQuery>) -> AppResult<Json<serde_json::Value>> {
    state.rate_limiter.acquire("spotify").await?;
    let token = ensure_token(&state).await?;

    let cache_key = format!("spotify:search:{}:{}", q.q, q.type_);
    let http = state.http.clone();
    let q_owned = q.q.clone();
    let t_owned = q.type_.clone();

    let raw = state
        .single_flight
        .do_once(&cache_key, move || async move {
            spotify::search(&http, &token, &q_owned, &t_owned).await
        })
        .await?;

    Ok(Json(raw))
}

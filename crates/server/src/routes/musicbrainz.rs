use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_providers::musicbrainz;

use crate::{
    db::entities::{
        musicbrainz_artists, musicbrainz_recordings, musicbrainz_releases, MusicbrainzArtists, MusicbrainzRecordings,
        MusicbrainzReleases,
    },
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/artist/:mbid", get(get_artist))
        .route("/release/:mbid", get(get_release))
        .route("/recording/:mbid", get(get_recording))
        .route("/search", get(get_search))
}

const DEFAULT_USER_AGENT: &str = "tokimo-server/0.1 (https://github.com/tokimo-lab/tokimo-server)";

fn user_agent(state: &AppState) -> String {
    state
        .config
        .musicbrainz_user_agent
        .clone()
        .unwrap_or_else(|| DEFAULT_USER_AGENT.to_string())
}

async fn get_artist(State(state): State<AppState>, Path(mbid): Path<String>) -> AppResult<Json<serde_json::Value>> {
    if let Some(row) = MusicbrainzArtists::find_by_id(mbid.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(Json(row.raw_json));
    }

    state.rate_limiter.acquire("musicbrainz").await?;

    let cache_key = format!("musicbrainz:artist:{}", mbid);
    let http = state.http.clone();
    let db = state.db.clone();
    let ua = user_agent(&state);
    let mbid_clone = mbid.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = MusicbrainzArtists::find_by_id(mbid_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = musicbrainz::fetch_artist(&http, &ua, &mbid_clone).await?;

            let am = musicbrainz_artists::ActiveModel {
                mbid: Set(mbid_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            MusicbrainzArtists::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json))
}

async fn get_release(State(state): State<AppState>, Path(mbid): Path<String>) -> AppResult<Json<serde_json::Value>> {
    if let Some(row) = MusicbrainzReleases::find_by_id(mbid.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(Json(row.raw_json));
    }

    state.rate_limiter.acquire("musicbrainz").await?;

    let cache_key = format!("musicbrainz:release:{}", mbid);
    let http = state.http.clone();
    let db = state.db.clone();
    let ua = user_agent(&state);
    let mbid_clone = mbid.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = MusicbrainzReleases::find_by_id(mbid_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = musicbrainz::fetch_release(&http, &ua, &mbid_clone).await?;

            let am = musicbrainz_releases::ActiveModel {
                mbid: Set(mbid_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            MusicbrainzReleases::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json))
}

async fn get_recording(State(state): State<AppState>, Path(mbid): Path<String>) -> AppResult<Json<serde_json::Value>> {
    if let Some(row) = MusicbrainzRecordings::find_by_id(mbid.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(Json(row.raw_json));
    }

    state.rate_limiter.acquire("musicbrainz").await?;

    let cache_key = format!("musicbrainz:recording:{}", mbid);
    let http = state.http.clone();
    let db = state.db.clone();
    let ua = user_agent(&state);
    let mbid_clone = mbid.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = MusicbrainzRecordings::find_by_id(mbid_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = musicbrainz::fetch_recording(&http, &ua, &mbid_clone).await?;

            let am = musicbrainz_recordings::ActiveModel {
                mbid: Set(mbid_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            MusicbrainzRecordings::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(rename = "type")]
    pub type_: String,
}

/// Search is not persisted (high cardinality query strings); we only apply
/// the rate limiter + single-flight to coalesce identical concurrent queries.
async fn get_search(State(state): State<AppState>, Query(q): Query<SearchQuery>) -> AppResult<Json<serde_json::Value>> {
    state.rate_limiter.acquire("musicbrainz").await?;

    let cache_key = format!("musicbrainz:search:{}:{}", q.type_, q.q);
    let http = state.http.clone();
    let ua = user_agent(&state);
    let q_owned = q.q.clone();
    let t_owned = q.type_.clone();

    let raw = state
        .single_flight
        .do_once(&cache_key, move || async move {
            musicbrainz::search(&http, &ua, &t_owned, &q_owned).await
        })
        .await?;

    Ok(Json(raw))
}

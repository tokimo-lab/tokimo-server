use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use serde_json::json;
use tokimo_core::CoreError;
use tokimo_providers::itunes;

use crate::{
    db::entities::{itunes_cache, ItunesCache},
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/album-cover", get(get_album_cover))
}

#[derive(Debug, Deserialize)]
struct AlbumCoverQuery {
    artist: Option<String>,
    album: Option<String>,
}

async fn get_album_cover(Query(params): Query<AlbumCoverQuery>, State(state): State<AppState>) -> AppResult<Response> {
    let artist = params.artist.as_deref().unwrap_or("").trim();
    let album = params.album.as_deref().unwrap_or("").trim();

    if artist.is_empty() || album.is_empty() {
        return Err(AppError::BadRequest("artist and album required".to_string()));
    }

    let artist_lower = artist.to_lowercase();
    let album_lower = album.to_lowercase();
    let cache_key = format!("itunes:cover:{}|{}", artist_lower, album_lower);

    let ttl_seconds = state.provider_ttl("itunes").await;
    if let Some(row) = ItunesCache::find_by_id(cache_key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        if is_fresh(row.fetched_at, ttl_seconds) {
            return Ok(cache_hit(Json(json!({
                "cover_url": pick_cover_url(&row.raw_json, &album_lower),
                "raw": row.raw_json
            }))));
        }
    }

    state.rate_limiter.acquire("itunes").await?;

    let sf_bucket = chrono::Utc::now().timestamp() / ttl_seconds.max(1);
    let cache_key_sf = format!("{}:{}", cache_key, sf_bucket);
    let http = state.http.clone();
    let db = state.db.clone();
    let artist_owned = artist.to_string();
    let album_owned = album.to_string();
    let cache_key_clone = cache_key.clone();

    let raw = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = ItunesCache::find_by_id(cache_key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?
            {
                if is_fresh(row.fetched_at, ttl_seconds) {
                    return Ok(row.raw_json);
                }
            }

            let raw = itunes::search_album_cover(&http, &artist_owned, &album_owned).await?;

            let am = itunes_cache::ActiveModel {
                cache_key: Set(cache_key_clone),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            ItunesCache::insert(am)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(itunes_cache::Column::CacheKey)
                        .update_columns([itunes_cache::Column::RawJson, itunes_cache::Column::FetchedAt])
                        .to_owned(),
                )
                .exec(&db)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(json!({
        "cover_url": pick_cover_url(&raw, &album_lower),
        "raw": raw
    }))
    .into_response())
}

fn is_fresh(fetched_at: chrono::DateTime<chrono::FixedOffset>, ttl_seconds: i64) -> bool {
    chrono::Utc::now().signed_duration_since(fetched_at) < chrono::Duration::seconds(ttl_seconds)
}

fn pick_cover_url(raw: &serde_json::Value, album: &str) -> Option<String> {
    let results = raw.get("results")?.as_array()?;
    let album_lower = album.to_lowercase();

    for item in results {
        if let Some(collection_name) = item.get("collectionName").and_then(|v| v.as_str()) {
            let collection_lower = collection_name.to_lowercase();
            if collection_lower.contains(&album_lower) || album_lower.contains(&collection_lower) {
                if let Some(artwork_url) = item.get("artworkUrl100").and_then(|v| v.as_str()) {
                    let high_res = artwork_url
                        .replace("100x100bb", "3000x3000bb")
                        .replace("/100x100", "/3000x3000");
                    return Some(high_res);
                }
            }
        }
    }

    None
}

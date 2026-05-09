use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_core::CoreError;
use tokimo_providers::bing_wallpaper;

use crate::{
    db::entities::{bing_wallpaper_cache, BingWallpaperCache},
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/wallpaper", get(get_wallpaper))
}

#[derive(Deserialize)]
pub struct WallpaperQuery {
    pub mkt: Option<String>,
    pub n: Option<u8>,
    pub idx: Option<u8>,
}

async fn get_wallpaper(State(state): State<AppState>, Query(q): Query<WallpaperQuery>) -> AppResult<Response> {
    let ttl_seconds = state.provider_ttl("bing").await;
    let mkt = q.mkt.unwrap_or_else(|| "zh-CN".to_string());
    if !bing_wallpaper::is_valid_market(&mkt) {
        return Err(AppError::BadRequest(format!(
            "invalid mkt '{mkt}', expected one of {:?}",
            bing_wallpaper::ALLOWED_MARKETS
        )));
    }
    let n = q.n.unwrap_or(1);
    if !(1..=8).contains(&n) {
        return Err(AppError::BadRequest("n must be between 1 and 8".to_string()));
    }
    let idx = q.idx.unwrap_or(0);
    if idx > 7 {
        return Err(AppError::BadRequest("idx must be between 0 and 7".to_string()));
    }

    let key = bing_wallpaper::cache_key(&mkt, n, idx);

    if let Some(row) = BingWallpaperCache::find_by_id(key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        if is_fresh(row.fetched_at, ttl_seconds) {
            return Ok(cache_hit(Json(row.raw_json)));
        }
    }

    state.rate_limiter.acquire("bing_wallpaper").await?;

    let sf_bucket = chrono::Utc::now().timestamp() / ttl_seconds.max(1);
    let cache_key_sf = format!("{key}:{sf_bucket}");
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();
    let mkt_clone = mkt.clone();

    let raw = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = BingWallpaperCache::find_by_id(key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?
            {
                if is_fresh(row.fetched_at, ttl_seconds) {
                    return Ok(row.raw_json);
                }
            }

            let raw = bing_wallpaper::fetch_wallpapers(&http, &mkt_clone, n, idx).await?;

            let am = bing_wallpaper_cache::ActiveModel {
                cache_key: Set(key_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            BingWallpaperCache::insert(am)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(bing_wallpaper_cache::Column::CacheKey)
                        .update_columns([
                            bing_wallpaper_cache::Column::RawJson,
                            bing_wallpaper_cache::Column::FetchedAt,
                        ])
                        .to_owned(),
                )
                .exec(&db)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw).into_response())
}

fn is_fresh(fetched_at: chrono::DateTime<chrono::FixedOffset>, ttl_seconds: i64) -> bool {
    chrono::Utc::now().signed_duration_since(fetched_at) < chrono::Duration::seconds(ttl_seconds)
}

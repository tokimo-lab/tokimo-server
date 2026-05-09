use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use tokimo_core::CoreError;
use tokimo_providers::zenquotes;

use crate::{
    db::entities::{zenquotes_cache, ZenquotesCache},
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

const CACHE_KEY: &str = "zenquotes:random";

pub fn routes() -> Router<AppState> {
    Router::new().route("/random", get(get_random))
}

async fn get_random(State(state): State<AppState>) -> AppResult<Response> {
    let ttl_seconds = state.provider_ttl("zenquotes").await;
    if let Some(row) = ZenquotesCache::find_by_id(CACHE_KEY.to_string())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        if is_fresh(row.fetched_at, ttl_seconds) {
            return Ok(cache_hit(Json(row.raw_json)));
        }
    }

    state.rate_limiter.acquire("zenquotes").await?;

    let sf_bucket = chrono::Utc::now().timestamp() / ttl_seconds.max(1);
    let cache_key_sf = format!("{CACHE_KEY}:{sf_bucket}");
    let http = state.http.clone();
    let db = state.db.clone();

    let raw = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = ZenquotesCache::find_by_id(CACHE_KEY.to_string())
                .one(&db)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?
            {
                if is_fresh(row.fetched_at, ttl_seconds) {
                    return Ok(row.raw_json);
                }
            }

            let raw = zenquotes::fetch_random(&http).await?;

            let am = zenquotes_cache::ActiveModel {
                cache_key: Set(CACHE_KEY.to_string()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            ZenquotesCache::insert(am)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(zenquotes_cache::Column::CacheKey)
                        .update_columns([zenquotes_cache::Column::RawJson, zenquotes_cache::Column::FetchedAt])
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

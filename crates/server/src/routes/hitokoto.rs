use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_core::CoreError;
use tokimo_providers::hitokoto;

use crate::{
    db::entities::{hitokoto_cache, HitokotoCache},
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/sentence", get(get_sentence))
}

#[derive(Deserialize)]
pub struct SentenceQuery {
    pub c: Option<String>,
}

async fn get_sentence(State(state): State<AppState>, Query(q): Query<SentenceQuery>) -> AppResult<Response> {
    let ttl_seconds = state.provider_ttl("hitokoto").await;
    let c = match q.c.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(cat) => {
            if !hitokoto::is_valid_category(cat) {
                return Err(AppError::BadRequest(format!("invalid category '{cat}', expected a-l")));
            }
            Some(cat.to_string())
        }
        None => None,
    };

    let key = hitokoto::cache_key(c.as_deref());

    if let Some(row) = HitokotoCache::find_by_id(key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        if is_fresh(row.fetched_at, ttl_seconds) {
            return Ok(cache_hit(Json(row.raw_json)));
        }
    }

    state.rate_limiter.acquire("hitokoto").await?;

    let sf_bucket = chrono::Utc::now().timestamp() / ttl_seconds.max(1);
    let cache_key_sf = format!("{key}:{sf_bucket}");
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();
    let c_clone = c.clone();

    let raw = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = HitokotoCache::find_by_id(key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?
            {
                if is_fresh(row.fetched_at, ttl_seconds) {
                    return Ok(row.raw_json);
                }
            }

            let raw = hitokoto::fetch_sentence(&http, c_clone.as_deref()).await?;

            let am = hitokoto_cache::ActiveModel {
                cache_key: Set(key_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            HitokotoCache::insert(am)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(hitokoto_cache::Column::CacheKey)
                        .update_columns([hitokoto_cache::Column::RawJson, hitokoto_cache::Column::FetchedAt])
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

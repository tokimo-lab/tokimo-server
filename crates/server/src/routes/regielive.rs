use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_core::CoreError;
use tokimo_providers::regielive;

use crate::{
    db::entities::{regielive_cache, RegieliveCache},
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

const CACHE_TTL_SECONDS: i64 = 12 * 60 * 60; // 12h

pub fn routes() -> Router<AppState> {
    Router::new().route("/search", get(get_search))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub nume: String,
}

async fn get_search(State(state): State<AppState>, Query(q): Query<SearchQuery>) -> AppResult<Response> {
    let nume = q.nume.trim().to_string();
    if nume.is_empty() {
        return Err(AppError::BadRequest("nume must not be empty".to_string()));
    }

    let key = regielive::cache_key(&nume);

    if let Some(row) = RegieliveCache::find_by_id(key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        if is_fresh(row.fetched_at) {
            return Ok(cache_hit(Json(row.raw_json)));
        }
    }

    state.rate_limiter.acquire("regielive").await?;

    let sf_bucket = chrono::Utc::now().timestamp() / CACHE_TTL_SECONDS;
    let cache_key_sf = format!("{key}:{sf_bucket}");
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();
    let nume_clone = nume.clone();

    let raw = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = RegieliveCache::find_by_id(key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?
            {
                if is_fresh(row.fetched_at) {
                    return Ok(row.raw_json);
                }
            }

            let raw = regielive::search(&http, &nume_clone).await?;

            let am = regielive_cache::ActiveModel {
                cache_key: Set(key_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            RegieliveCache::insert(am)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(regielive_cache::Column::CacheKey)
                        .update_columns([regielive_cache::Column::RawJson, regielive_cache::Column::FetchedAt])
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

fn is_fresh(fetched_at: chrono::DateTime<chrono::FixedOffset>) -> bool {
    chrono::Utc::now().signed_duration_since(fetched_at) < chrono::Duration::seconds(CACHE_TTL_SECONDS)
}

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_core::CoreError;
use tokimo_providers::animetosho;

use crate::{
    db::entities::{animetosho_cache, AnimetoshoCache},
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/search", get(get_search))
        .route("/torrent", get(get_torrent))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Deserialize)]
pub struct TorrentQuery {
    pub id: u64,
}

async fn get_search(State(state): State<AppState>, Query(q): Query<SearchQuery>) -> AppResult<Response> {
    let query = q.q.trim().to_string();
    if query.is_empty() {
        return Err(AppError::BadRequest("q must not be empty".to_string()));
    }
    let key = animetosho::search_cache_key(&query);
    let query_clone = query.clone();
    fetch_with_cache(state, key, move |http| async move {
        animetosho::search(&http, &query_clone).await
    })
    .await
}

async fn get_torrent(State(state): State<AppState>, Query(q): Query<TorrentQuery>) -> AppResult<Response> {
    let id = q.id;
    let key = animetosho::torrent_cache_key(id);
    fetch_with_cache(
        state,
        key,
        move |http| async move { animetosho::torrent(&http, id).await },
    )
    .await
}

async fn fetch_with_cache<F, Fut>(state: AppState, key: String, fetch: F) -> AppResult<Response>
where
    F: FnOnce(reqwest::Client) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = tokimo_core::CoreResult<serde_json::Value>> + Send,
{
    let ttl_seconds = state.provider_ttl("animetosho").await;

    if let Some(row) = AnimetoshoCache::find_by_id(key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        if is_fresh(row.fetched_at, ttl_seconds) {
            return Ok(cache_hit(Json(row.raw_json)));
        }
    }

    state.rate_limiter.acquire("animetosho").await?;

    let sf_bucket = chrono::Utc::now().timestamp() / ttl_seconds.max(1);
    let cache_key_sf = format!("{key}:{sf_bucket}");
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();

    let raw = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = AnimetoshoCache::find_by_id(key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?
            {
                if is_fresh(row.fetched_at, ttl_seconds) {
                    return Ok(row.raw_json);
                }
            }

            let raw = fetch(http).await?;

            let am = animetosho_cache::ActiveModel {
                cache_key: Set(key_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            AnimetoshoCache::insert(am)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(animetosho_cache::Column::CacheKey)
                        .update_columns([animetosho_cache::Column::RawJson, animetosho_cache::Column::FetchedAt])
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

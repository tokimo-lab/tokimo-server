use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_core::CoreError;
use tokimo_providers::gestdown;

use crate::{
    db::entities::{gestdown_cache, GestdownCache},
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

const CACHE_TTL_SECONDS: i64 = 12 * 60 * 60; // 12h

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/shows/search", get(get_shows_search))
        .route("/subtitles", get(get_subtitles))
}

#[derive(Deserialize)]
pub struct ShowsSearchQuery {
    pub title: String,
}

#[derive(Deserialize)]
pub struct SubtitlesQuery {
    pub show_id: String,
    pub season: u32,
    pub episode: u32,
    pub lang: String,
}

async fn get_shows_search(State(state): State<AppState>, Query(q): Query<ShowsSearchQuery>) -> AppResult<Response> {
    let title = q.title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::BadRequest("title must not be empty".to_string()));
    }
    let key = gestdown::shows_cache_key(&title);
    let title_clone = title.clone();
    fetch_with_cache(state, key, move |http| async move {
        gestdown::search_shows(&http, &title_clone).await
    })
    .await
}

async fn get_subtitles(State(state): State<AppState>, Query(q): Query<SubtitlesQuery>) -> AppResult<Response> {
    let show_id = q.show_id.trim().to_string();
    let lang = q.lang.trim().to_string();
    if show_id.is_empty() || lang.is_empty() {
        return Err(AppError::BadRequest("show_id and lang must not be empty".to_string()));
    }
    let key = gestdown::subs_cache_key(&show_id, q.season, q.episode, &lang);
    let show_id_clone = show_id.clone();
    let lang_clone = lang.clone();
    let season = q.season;
    let episode = q.episode;
    fetch_with_cache(state, key, move |http| async move {
        gestdown::get_subtitles(&http, &show_id_clone, season, episode, &lang_clone).await
    })
    .await
}

async fn fetch_with_cache<F, Fut>(state: AppState, key: String, fetch: F) -> AppResult<Response>
where
    F: FnOnce(reqwest::Client) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = tokimo_core::CoreResult<serde_json::Value>> + Send,
{
    if let Some(row) = GestdownCache::find_by_id(key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        if is_fresh(row.fetched_at) {
            return Ok(cache_hit(Json(row.raw_json)));
        }
    }

    state.rate_limiter.acquire("gestdown").await?;

    let sf_bucket = chrono::Utc::now().timestamp() / CACHE_TTL_SECONDS;
    let cache_key_sf = format!("{key}:{sf_bucket}");
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();

    let raw = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = GestdownCache::find_by_id(key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?
            {
                if is_fresh(row.fetched_at) {
                    return Ok(row.raw_json);
                }
            }

            let raw = fetch(http).await?;

            let am = gestdown_cache::ActiveModel {
                cache_key: Set(key_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            GestdownCache::insert(am)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(gestdown_cache::Column::CacheKey)
                        .update_columns([gestdown_cache::Column::RawJson, gestdown_cache::Column::FetchedAt])
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

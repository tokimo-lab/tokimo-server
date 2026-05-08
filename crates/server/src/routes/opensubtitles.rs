use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_core::CoreError;
use tokimo_providers::opensubtitles;

use crate::{
    db::entities::{opensubtitles_cache, OpensubtitlesCache},
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

const CACHE_TTL_SECONDS: i64 = 6 * 60 * 60; // 6h

pub fn routes() -> Router<AppState> {
    Router::new().route("/search", get(get_search))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<String>,
    pub languages: Option<String>,
}

async fn get_search(State(state): State<AppState>, Query(q): Query<SearchQuery>) -> AppResult<Response> {
    let api_key = state.config.opensubtitles_api_key.clone().ok_or_else(|| {
        AppError::Core(CoreError::Provider(
            "opensubtitles: OPENSUBTITLES_API_KEY is not configured".into(),
        ))
    })?;

    let query = q.query.as_deref().map(str::trim).unwrap_or("").to_string();
    let imdb = q.imdb_id.as_deref().map(str::trim).unwrap_or("").to_string();
    let tmdb = q.tmdb_id.as_deref().map(str::trim).unwrap_or("").to_string();
    let langs = q.languages.as_deref().map(str::trim).unwrap_or("").to_string();

    if query.is_empty() && imdb.is_empty() {
        return Err(AppError::BadRequest(
            "at least one of query / imdb_id is required".to_string(),
        ));
    }

    let imdb_normalized = opensubtitles::normalize_imdb(&imdb).to_string();
    let key = opensubtitles::cache_key(&query, &imdb_normalized, &tmdb, &langs);

    if let Some(row) = OpensubtitlesCache::find_by_id(key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        if is_fresh(row.fetched_at) {
            return Ok(cache_hit(Json(row.raw_json)));
        }
    }

    state.rate_limiter.acquire("opensubtitles").await?;

    let sf_bucket = chrono::Utc::now().timestamp() / CACHE_TTL_SECONDS;
    let cache_key_sf = format!("{key}:{sf_bucket}");
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();
    let api_key_clone = api_key.clone();
    let query_owned = query.clone();
    let imdb_owned = imdb.clone();
    let tmdb_owned = tmdb.clone();
    let langs_owned = langs.clone();

    let raw = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = OpensubtitlesCache::find_by_id(key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?
            {
                if is_fresh(row.fetched_at) {
                    return Ok(row.raw_json);
                }
            }

            let raw = opensubtitles::search(
                &http,
                &api_key_clone,
                Some(query_owned.as_str()),
                Some(imdb_owned.as_str()),
                Some(tmdb_owned.as_str()),
                Some(langs_owned.as_str()),
            )
            .await?;

            let am = opensubtitles_cache::ActiveModel {
                cache_key: Set(key_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            OpensubtitlesCache::insert(am)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(opensubtitles_cache::Column::CacheKey)
                        .update_columns([
                            opensubtitles_cache::Column::RawJson,
                            opensubtitles_cache::Column::FetchedAt,
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

fn is_fresh(fetched_at: chrono::DateTime<chrono::FixedOffset>) -> bool {
    chrono::Utc::now().signed_duration_since(fetched_at) < chrono::Duration::seconds(CACHE_TTL_SECONDS)
}

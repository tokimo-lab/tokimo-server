use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_core::CoreError;
use tokimo_providers::shooter;

use crate::{
    db::entities::{shooter_cache, ShooterCache},
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

const CACHE_TTL_SECONDS: i64 = 12 * 60 * 60; // 12h

pub fn routes() -> Router<AppState> {
    Router::new().route("/search", get(get_search))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub filehash: String,
    #[serde(default)]
    pub pathinfo: Option<String>,
    #[serde(default)]
    pub lang: Option<String>,
}

async fn get_search(State(state): State<AppState>, Query(q): Query<SearchQuery>) -> AppResult<Response> {
    let file_hash = q.filehash.trim().to_string();
    if file_hash.is_empty() {
        return Err(AppError::BadRequest("filehash must not be empty".to_string()));
    }
    let path_info = q.pathinfo.unwrap_or_default().trim().to_string();
    let lang = {
        let raw = q.lang.unwrap_or_default();
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            "chn".to_string()
        } else {
            trimmed.to_string()
        }
    };

    let key = shooter::cache_key(&file_hash, &path_info, &lang);

    if let Some(row) = ShooterCache::find_by_id(key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        if is_fresh(row.fetched_at) {
            return Ok(cache_hit(Json(row.raw_json)));
        }
    }

    state.rate_limiter.acquire("shooter").await?;

    let sf_bucket = chrono::Utc::now().timestamp() / CACHE_TTL_SECONDS;
    let cache_key_sf = format!("{key}:{sf_bucket}");
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();
    let file_hash_clone = file_hash.clone();
    let path_info_clone = path_info.clone();
    let lang_clone = lang.clone();

    let raw = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = ShooterCache::find_by_id(key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?
            {
                if is_fresh(row.fetched_at) {
                    return Ok(row.raw_json);
                }
            }

            let raw = shooter::search(&http, &file_hash_clone, &path_info_clone, &lang_clone).await?;

            let am = shooter_cache::ActiveModel {
                cache_key: Set(key_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            ShooterCache::insert(am)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(shooter_cache::Column::CacheKey)
                        .update_columns([shooter_cache::Column::RawJson, shooter_cache::Column::FetchedAt])
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

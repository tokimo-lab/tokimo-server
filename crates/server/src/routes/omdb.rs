use std::time::Instant;

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_providers::omdb;

use crate::{
    db::entities::{omdb_titles, OmdbTitles},
    metrics::cache_hit_response,
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/title/:imdb_id", get(get_title))
        .route("/search", get(get_search))
}

fn require_key(state: &AppState) -> AppResult<String> {
    state
        .config
        .omdb_api_key
        .clone()
        .ok_or_else(|| AppError::Internal("OMDB_API_KEY not configured".into()))
}

async fn get_title(State(state): State<AppState>, Path(imdb_id): Path<String>) -> AppResult<Response> {
    let started = Instant::now();
    if let Some(row) = OmdbTitles::find_by_id(imdb_id.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(cache_hit_response(&state, "omdb", started, Json(row.raw_json)));
    }

    let api_key = require_key(&state)?;
    state.rate_limiter.acquire("omdb").await?;

    let cache_key = format!("omdb:title:{}", imdb_id);
    let http = state.http.clone();
    let db = state.db.clone();
    let imdb_id_clone = imdb_id.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = OmdbTitles::find_by_id(imdb_id_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = match omdb::fetch_title(&http, &api_key, &imdb_id_clone).await? {
                Some(v) => v,
                None => return Err(tokimo_core::CoreError::NotFound),
            };

            let am = omdb_titles::ActiveModel {
                imdb_id: Set(imdb_id_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            OmdbTitles::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json).into_response())
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub y: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

/// Search is not persisted (high cardinality query strings); we only apply
/// the rate limiter + single-flight to coalesce identical concurrent queries.
async fn get_search(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> AppResult<Json<omdb::OmdbSearchResponse>> {
    let api_key = require_key(&state)?;
    state.rate_limiter.acquire("omdb").await?;

    let cache_key = format!(
        "omdb:search:{}:{}:{}",
        q.q,
        q.y.as_deref().unwrap_or(""),
        q.type_.as_deref().unwrap_or("")
    );
    let http = state.http.clone();
    let q_owned = q.q.clone();
    let y_owned = q.y.clone();
    let t_owned = q.type_.clone();

    let resp = state
        .single_flight
        .do_once(&cache_key, move || async move {
            omdb::search(&http, &api_key, &q_owned, y_owned.as_deref(), t_owned.as_deref()).await
        })
        .await?;

    Ok(Json(resp))
}

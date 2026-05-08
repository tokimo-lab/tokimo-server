use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_providers::geocoding;

use crate::{
    db::entities::{geocoding_results, GeocodingResults},
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/forward", get(get_forward))
        .route("/reverse", get(get_reverse))
}

#[derive(Deserialize)]
pub struct ForwardQuery {
    pub q: String,
    pub lang: Option<String>,
    #[serde(default = "default_count")]
    pub count: u8,
}

fn default_count() -> u8 {
    10
}

async fn get_forward(State(state): State<AppState>, Query(q): Query<ForwardQuery>) -> AppResult<Response> {
    let key = geocoding::forward_cache_key(&q.q, q.lang.as_deref(), q.count);

    if let Some(row) = GeocodingResults::find_by_id(key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(cache_hit(Json(row.raw_json)));
    }

    state.rate_limiter.acquire("geocoding").await?;
    state.rate_limiter.acquire("openmeteo").await?;

    let cache_key_sf = format!("geocoding:fwd:{}", key);
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();
    let name = q.q.clone();
    let lang = q.lang.clone();
    let count = q.count;

    let raw_json = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = GeocodingResults::find_by_id(key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = geocoding::fetch_forward(&http, &name, lang.as_deref(), count).await?;

            let am = geocoding_results::ActiveModel {
                cache_key: Set(key_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            GeocodingResults::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json).into_response())
}

#[derive(Deserialize)]
pub struct ReverseQuery {
    pub lat: f64,
    pub lon: f64,
    pub lang: Option<String>,
}

async fn get_reverse(State(state): State<AppState>, Query(q): Query<ReverseQuery>) -> AppResult<Response> {
    let key = geocoding::reverse_cache_key(q.lat, q.lon, q.lang.as_deref());

    if let Some(row) = GeocodingResults::find_by_id(key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(cache_hit(Json(row.raw_json)));
    }

    state.rate_limiter.acquire("geocoding").await?;
    state.rate_limiter.acquire("nominatim").await?;

    let cache_key_sf = format!("geocoding:rev:{}", key);
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();
    let lat = q.lat;
    let lon = q.lon;
    let lang = q.lang.clone();
    let user_agent = state
        .config
        .nominatim_user_agent
        .clone()
        .unwrap_or_else(|| "tokimo-server/0.1 (geocoding aggregator)".to_string());

    let raw_json = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = GeocodingResults::find_by_id(key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = geocoding::fetch_reverse(&http, lat, lon, lang.as_deref(), &user_agent).await?;

            let am = geocoding_results::ActiveModel {
                cache_key: Set(key_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            GeocodingResults::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json).into_response())
}

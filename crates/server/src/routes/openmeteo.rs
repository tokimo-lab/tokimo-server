use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_providers::open_meteo;

use crate::{
    db::entities::{openmeteo_forecasts, OpenmeteoForecasts},
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/forecast", get(get_forecast))
        .route("/air-quality", get(get_air_quality))
}

#[derive(Deserialize)]
pub struct ForecastQuery {
    pub lat: f64,
    pub lon: f64,
    #[serde(default = "default_days")]
    pub days: u8,
}

fn default_days() -> u8 {
    7
}

async fn get_forecast(
    State(state): State<AppState>,
    Query(q): Query<ForecastQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let key = open_meteo::forecast_cache_key(q.lat, q.lon, q.days);

    if let Some(row) = OpenmeteoForecasts::find_by_id(key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(Json(row.raw_json));
    }

    state.rate_limiter.acquire("openmeteo").await?;

    let cache_key_sf = format!("openmeteo:forecast:{}", key);
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();
    let lat = q.lat;
    let lon = q.lon;
    let days = q.days;

    let raw_json = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = OpenmeteoForecasts::find_by_id(key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = open_meteo::fetch_forecast(&http, lat, lon, days).await?;

            let am = openmeteo_forecasts::ActiveModel {
                cache_key: Set(key_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            OpenmeteoForecasts::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json))
}

#[derive(Deserialize)]
pub struct AirQualityQuery {
    pub lat: f64,
    pub lon: f64,
}

async fn get_air_quality(
    State(state): State<AppState>,
    Query(q): Query<AirQualityQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let key = open_meteo::air_quality_cache_key(q.lat, q.lon);

    if let Some(row) = OpenmeteoForecasts::find_by_id(key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(Json(row.raw_json));
    }

    state.rate_limiter.acquire("openmeteo").await?;

    let cache_key_sf = format!("openmeteo:air:{}", key);
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();
    let lat = q.lat;
    let lon = q.lon;

    let raw_json = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = OpenmeteoForecasts::find_by_id(key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = open_meteo::fetch_air_quality(&http, lat, lon).await?;

            let am = openmeteo_forecasts::ActiveModel {
                cache_key: Set(key_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            OpenmeteoForecasts::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json))
}

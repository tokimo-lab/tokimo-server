//! Open-Meteo weather adapter — copied + adapted from
//! tokimo/packages/rust-client-api/src/open_meteo.rs
//!
//! Differences from the upstream client:
//! - Returns raw JSON instead of typed DTOs; tokimo-server's DB cache stores
//!   the unparsed body and downstream consumers parse what they need.
//! - Forecast hours kept at 48; daily horizon is parameterized by `days`.

use serde_json::Value;
use tokimo_core::{CoreError, CoreResult};

const FORECAST_BASE: &str = "https://api.open-meteo.com/v1/forecast";
const AIR_QUALITY_BASE: &str = "https://air-quality-api.open-meteo.com/v1/air-quality";

/// Stable cache key for the forecast endpoint.
///
/// Coordinates are rounded to 2 decimal places (~1.1km granularity), which is
/// far below typical user-perceptible accuracy and keeps the cache hit rate
/// high for nearby requests.
pub fn forecast_cache_key(lat: f64, lon: f64, days: u8) -> String {
    format!("{:.2},{:.2}|{}", lat, lon, days)
}

/// Stable cache key for the air-quality endpoint.
pub fn air_quality_cache_key(lat: f64, lon: f64) -> String {
    format!("aq|{:.2},{:.2}", lat, lon)
}

/// Fetch the standard forecast bundle (current + 48 hourly + N daily).
pub async fn fetch_forecast(http: &reqwest::Client, lat: f64, lon: f64, days: u8) -> CoreResult<Value> {
    let url = format!(
        "{FORECAST_BASE}\
         ?latitude={lat}&longitude={lon}\
         &current=temperature_2m,relative_humidity_2m,apparent_temperature,is_day,\
         precipitation,rain,snowfall,weather_code,cloud_cover,pressure_msl,\
         surface_pressure,wind_speed_10m,wind_direction_10m,wind_gusts_10m\
         &hourly=temperature_2m,relative_humidity_2m,apparent_temperature,\
         precipitation_probability,precipitation,rain,snowfall,weather_code,\
         cloud_cover,visibility,wind_speed_10m,wind_direction_10m,pressure_msl,is_day\
         &daily=weather_code,temperature_2m_max,temperature_2m_min,\
         sunrise,sunset,precipitation_sum,rain_sum,snowfall_sum,\
         precipitation_probability_max,wind_speed_10m_max,uv_index_max\
         &wind_speed_unit=ms&timezone=auto&forecast_hours=48&forecast_days={days}"
    );
    get_json(http, &url, "Open-Meteo forecast").await
}

/// Fetch the air-quality endpoint (current pollutant readings + AQI).
pub async fn fetch_air_quality(http: &reqwest::Client, lat: f64, lon: f64) -> CoreResult<Value> {
    let url = format!(
        "{AIR_QUALITY_BASE}\
         ?latitude={lat}&longitude={lon}\
         &current=european_aqi,us_aqi,pm10,pm2_5,carbon_monoxide,\
         nitrogen_dioxide,sulphur_dioxide,ozone,dust,uv_index\
         &timezone=auto"
    );
    get_json(http, &url, "Open-Meteo air-quality").await
}

async fn get_json(http: &reqwest::Client, url: &str, label: &str) -> CoreResult<Value> {
    let resp = http.get(url).send().await.map_err(CoreError::Upstream)?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(CoreError::Provider(format!("{label} returned status {status}: {body}")));
    }
    resp.json::<Value>().await.map_err(CoreError::Upstream)
}

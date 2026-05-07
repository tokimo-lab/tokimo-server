//! Geocoding aggregator — copied + adapted from
//! tokimo/packages/rust-client-api/src/geocoding.rs
//!
//! Differences from the upstream client:
//! - Drops commercial provider integrations (Amap/QQMap/Tianditu/Mapbox/MapTiler)
//!   that require API keys.
//! - Forward geocoding uses Open-Meteo's free Geocoding API.
//! - Reverse geocoding falls back to Nominatim (OSM).
//! - Returns raw JSON for caller-side parsing; tokimo-server stores the body
//!   verbatim in the DB cache.

use serde_json::Value;
use sha2::{Digest, Sha256};
use tokimo_core::{CoreError, CoreResult};

const OPEN_METEO_GEOCODING: &str = "https://geocoding-api.open-meteo.com/v1/search";
const NOMINATIM_REVERSE: &str = "https://nominatim.openstreetmap.org/reverse";

/// Stable cache key for a forward geocoding request.
pub fn forward_cache_key(name: &str, lang: Option<&str>, count: u8) -> String {
    let raw = format!("fwd|{}|{}|{}", name.trim(), lang.unwrap_or(""), count);
    let mut h = Sha256::new();
    h.update(raw.as_bytes());
    format!("{:x}", h.finalize())
}

/// Stable cache key for a reverse geocoding request.
pub fn reverse_cache_key(lat: f64, lon: f64, lang: Option<&str>) -> String {
    let raw = format!("rev|{:.5},{:.5}|{}", lat, lon, lang.unwrap_or(""));
    let mut h = Sha256::new();
    h.update(raw.as_bytes());
    format!("{:x}", h.finalize())
}

/// Forward geocoding via Open-Meteo. Returns raw JSON.
pub async fn fetch_forward(http: &reqwest::Client, name: &str, lang: Option<&str>, count: u8) -> CoreResult<Value> {
    let count = count.clamp(1, 20);
    let mut url = format!(
        "{OPEN_METEO_GEOCODING}?name={}&count={}&format=json",
        urlencoding::encode(name),
        count
    );
    if let Some(l) = lang {
        url.push_str(&format!("&language={}", urlencoding::encode(l)));
    }
    get_json(http, &url, None, "Open-Meteo geocoding").await
}

/// Reverse geocoding via Nominatim. Returns raw JSON.
pub async fn fetch_reverse(
    http: &reqwest::Client,
    lat: f64,
    lon: f64,
    lang: Option<&str>,
    user_agent: &str,
) -> CoreResult<Value> {
    let mut url = format!("{NOMINATIM_REVERSE}?lat={}&lon={}&format=json", lat, lon);
    if let Some(l) = lang {
        url.push_str(&format!("&accept-language={}", urlencoding::encode(l)));
    }
    get_json(http, &url, Some(user_agent), "Nominatim reverse").await
}

async fn get_json(http: &reqwest::Client, url: &str, user_agent: Option<&str>, label: &str) -> CoreResult<Value> {
    let mut req = http.get(url);
    if let Some(ua) = user_agent {
        req = req.header(reqwest::header::USER_AGENT, ua);
    }
    let resp = req.send().await.map_err(CoreError::Upstream)?;
    if !resp.status().is_success() {
        return Err(CoreError::Provider(format!("{label} returned HTTP {}", resp.status())));
    }
    resp.json::<Value>().await.map_err(CoreError::Upstream)
}

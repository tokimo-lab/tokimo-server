//! Nominatim (OpenStreetMap) geocoding adapter — copied + adapted from
//! tokimo/packages/rust-client-api/src/nominatim.rs
//!
//! Differences from the upstream client:
//! - Returns raw JSON instead of typed `NominatimEntry` DTOs.
//! - User-Agent is required by Nominatim's usage policy and is supplied by
//!   the route handler from `AppConfig`.

use serde_json::Value;
use sha2::{Digest, Sha256};
use tokimo_core::{CoreError, CoreResult};

const BASE_URL: &str = "https://nominatim.openstreetmap.org";

/// Stable cache key — sha256 of the normalized request shape.
pub fn cache_key(kind: &str, args: &[(&str, &str)]) -> String {
    let mut h = Sha256::new();
    h.update(kind.as_bytes());
    h.update(b"|");
    for (k, v) in args {
        h.update(k.as_bytes());
        h.update(b"=");
        h.update(v.as_bytes());
        h.update(b"&");
    }
    hex::encode(h.finalize())
}

/// Forward search: `q` → list of matching places (raw JSON array).
pub async fn search(http: &reqwest::Client, user_agent: &str, q: &str, limit: u8, lang: &str) -> CoreResult<Value> {
    let url = format!(
        "{BASE_URL}/search?q={}&format=json&limit={}&accept-language={}&addressdetails=1",
        urlencoding::encode(q),
        limit,
        urlencoding::encode(lang),
    );
    get_json(http, user_agent, &url, "Nominatim search").await
}

/// Reverse geocode: `(lat, lon)` → place description (raw JSON object).
pub async fn reverse(http: &reqwest::Client, user_agent: &str, lat: f64, lon: f64, lang: &str) -> CoreResult<Value> {
    let url = format!(
        "{BASE_URL}/reverse?lat={:.6}&lon={:.6}&format=json&accept-language={}&addressdetails=1",
        lat,
        lon,
        urlencoding::encode(lang),
    );
    get_json(http, user_agent, &url, "Nominatim reverse").await
}

async fn get_json(http: &reqwest::Client, user_agent: &str, url: &str, label: &str) -> CoreResult<Value> {
    let resp = http
        .get(url)
        .header("User-Agent", user_agent)
        .send()
        .await
        .map_err(CoreError::Upstream)?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(CoreError::Provider(format!("{label} returned status {status}: {body}")));
    }
    resp.json::<Value>().await.map_err(CoreError::Upstream)
}

//! Deezer public API adapter — copied + adapted from
//! tokimo/packages/rust-client-api/src/metadata_providers/deezer.rs
//!
//! Differences from the upstream client:
//! - The original client only exposed `get_artist_photo` for one specific
//!   need; tokimo-server exposes the broader entity-detail + search
//!   endpoints we need server-side. Underlying transport (plain
//!   GET, no auth) is unchanged.
//! - `RequestCache` is replaced by tokimo-server's DB cache.

use tokimo_core::CoreResult;

use crate::common::http_get_json;

pub const DEEZER_API: &str = "https://api.deezer.com";

/// GET /track/{id}
pub async fn fetch_track(http: &reqwest::Client, id: i64) -> CoreResult<serde_json::Value> {
    let url = format!("{}/track/{}", DEEZER_API, id);
    http_get_json(http, &url).await
}

/// GET /album/{id}
pub async fn fetch_album(http: &reqwest::Client, id: i64) -> CoreResult<serde_json::Value> {
    let url = format!("{}/album/{}", DEEZER_API, id);
    http_get_json(http, &url).await
}

/// GET /artist/{id}
pub async fn fetch_artist(http: &reqwest::Client, id: i64) -> CoreResult<serde_json::Value> {
    let url = format!("{}/artist/{}", DEEZER_API, id);
    http_get_json(http, &url).await
}

/// GET /search?q={q}
pub async fn search(http: &reqwest::Client, q: &str) -> CoreResult<serde_json::Value> {
    let url = format!("{}/search?q={}", DEEZER_API, urlencoding::encode(q));
    http_get_json(http, &url).await
}

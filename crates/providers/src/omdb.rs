//! OMDb adapter — copied + adapted from
//! tokimo/packages/rust-client-api/src/metadata_providers/omdb.rs
//!
//! The original `OmdbClient` wraps an internal `RequestCache` and request()
//! helper; we drop the cache (single-flight + DB layer in tokimo-server
//! handles that) and expose pure async fetch fns returning raw JSON.

use serde::{Deserialize, Serialize};
use tokimo_core::{CoreError, CoreResult};

use crate::common::http_get_json;

pub const OMDB_BASE: &str = "https://www.omdbapi.com";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmdbSearchItem {
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Year")]
    pub year: String,
    #[serde(rename = "imdbID")]
    pub imdb_id: String,
    #[serde(rename = "Type")]
    pub media_type: String,
    #[serde(rename = "Poster")]
    pub poster: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmdbSearchResponse {
    #[serde(rename = "Search")]
    pub search: Option<Vec<OmdbSearchItem>>,
    #[serde(rename = "Response")]
    pub response: String,
    #[serde(rename = "Error")]
    pub error: Option<String>,
}

fn build_url(api_key: &str, params: &[(&str, &str)]) -> String {
    let mut url = format!("{}/?apikey={}", OMDB_BASE, urlencoding::encode(api_key));
    for (k, v) in params {
        url.push('&');
        url.push_str(&urlencoding::encode(k));
        url.push('=');
        url.push_str(&urlencoding::encode(v));
    }
    url
}

/// Get title detail by IMDb id; returns the raw JSON (or None when OMDb
/// reports `Response=False`, i.e. the title doesn't exist).
pub async fn fetch_title(
    http: &reqwest::Client,
    api_key: &str,
    imdb_id: &str,
) -> CoreResult<Option<serde_json::Value>> {
    let url = build_url(api_key, &[("i", imdb_id)]);
    let raw = http_get_json(http, &url).await?;
    if raw.get("Response").and_then(|v| v.as_str()) == Some("False") {
        return Ok(None);
    }
    Ok(Some(raw))
}

/// Search by title (`s`), optional year (`y`) and optional type filter
/// (`type`: movie | series | episode).
pub async fn search(
    http: &reqwest::Client,
    api_key: &str,
    query: &str,
    year: Option<&str>,
    media_type: Option<&str>,
) -> CoreResult<OmdbSearchResponse> {
    let mut params: Vec<(&str, &str)> = vec![("s", query)];
    if let Some(y) = year {
        params.push(("y", y));
    }
    if let Some(t) = media_type {
        params.push(("type", t));
    }
    let url = build_url(api_key, &params);
    let raw = http_get_json(http, &url).await?;
    serde_json::from_value(raw).map_err(|e| CoreError::Provider(format!("OMDb search parse error: {}", e)))
}

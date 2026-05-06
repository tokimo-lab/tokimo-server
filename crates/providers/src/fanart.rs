//! Fanart.tv adapter — copied + adapted from
//! tokimo/packages/rust-client-api/src/metadata_providers/fanart.rs
//!
//! Returns raw JSON; persistence + single-flight at the route layer.
//! Image URL selection helpers (pick_best / get_best_*) are intentionally
//! not ported — clients can do that selection themselves from the cached
//! JSON or we can add typed helpers on demand.

use serde_json::Value;
use tokimo_core::{CoreError, CoreResult};

use crate::common::http_get_json;

pub const FANART_BASE: &str = "https://webservice.fanart.tv/v3";

fn build_url(path: &str, api_key: &str, client_key: Option<&str>) -> String {
    let mut url = format!("{}{}?api_key={}", FANART_BASE, path, urlencoding::encode(api_key));
    if let Some(ck) = client_key {
        url.push_str("&client_key=");
        url.push_str(&urlencoding::encode(ck));
    }
    url
}

/// Fetch movie images bundle by TMDb id; returns Ok(None) on 404.
pub async fn fetch_movie_images(
    http: &reqwest::Client,
    api_key: &str,
    client_key: Option<&str>,
    tmdb_id: i64,
) -> CoreResult<Option<Value>> {
    let url = build_url(&format!("/movies/{}", tmdb_id), api_key, client_key);
    fetch_optional(http, &url).await
}

/// Fetch TV images bundle by TheTVDB id; returns Ok(None) on 404.
pub async fn fetch_tv_images(
    http: &reqwest::Client,
    api_key: &str,
    client_key: Option<&str>,
    tvdb_id: i64,
) -> CoreResult<Option<Value>> {
    let url = build_url(&format!("/tv/{}", tvdb_id), api_key, client_key);
    fetch_optional(http, &url).await
}

async fn fetch_optional(http: &reqwest::Client, url: &str) -> CoreResult<Option<Value>> {
    let resp = http.get(url).send().await.map_err(CoreError::Upstream)?;
    if resp.status().as_u16() == 404 {
        return Ok(None);
    }
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(CoreError::Provider(format!("Fanart API {status}: {body}")));
    }
    let v = resp.json::<Value>().await.map_err(CoreError::Upstream)?;
    Ok(Some(v))
}

// Re-export helper so callers that want raw HTTP can use it without
// reaching across crates. (Currently unused but kept symmetric with
// other adapters.)
#[allow(dead_code)]
async fn _http_get_json_anchor(http: &reqwest::Client, url: &str) -> CoreResult<Value> {
    http_get_json(http, url).await
}

//! Bing daily wallpaper adapter.
//!
//! Endpoint:
//!   `https://www.bing.com/HPImageArchive.aspx?format=js&idx={idx}&n={n}&mkt={mkt}`
//!
//! Returns `{images: [...], tooltips: {...}}`. The upstream `images[].url`
//! is a relative path like `/th?id=...`; this adapter rewrites it to an
//! absolute `https://www.bing.com/...` URL so callers don't have to.

use serde_json::Value;
use tokimo_core::{CoreError, CoreResult};

const BING_HOST: &str = "https://www.bing.com";
pub const ALLOWED_MARKETS: &[&str] = &["zh-CN", "en-US", "ja-JP"];

/// Build the cache key.
pub fn cache_key(mkt: &str, n: u8, idx: u8) -> String {
    format!("bing_wallpaper:{mkt}:{n}:{idx}")
}

pub fn is_valid_market(mkt: &str) -> bool {
    ALLOWED_MARKETS.contains(&mkt)
}

/// Fetch wallpapers and return raw JSON with absolute image URLs.
pub async fn fetch_wallpapers(http: &reqwest::Client, mkt: &str, n: u8, idx: u8) -> CoreResult<Value> {
    let url = format!("{BING_HOST}/HPImageArchive.aspx?format=js&idx={idx}&n={n}&mkt={mkt}");

    let resp = http.get(&url).send().await.map_err(CoreError::Upstream)?;
    let status = resp.status();
    if !status.is_success() {
        return Err(CoreError::Provider(format!("bing returned status {status}")));
    }

    let mut body: Value = resp.json().await.map_err(CoreError::Upstream)?;

    if let Some(images) = body.get_mut("images").and_then(Value::as_array_mut) {
        for img in images.iter_mut() {
            if let Some(rel) = img.get("url").and_then(Value::as_str) {
                if rel.starts_with('/') {
                    let absolute = format!("{BING_HOST}{rel}");
                    if let Some(obj) = img.as_object_mut() {
                        obj.insert("url".to_string(), Value::String(absolute));
                    }
                }
            }
        }
    }

    Ok(body)
}

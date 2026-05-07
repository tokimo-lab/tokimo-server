//! ASSRT subtitle provider — adapted from
//! tokimo/packages/rust-client-api/src/assrt.rs
//!
//! The upstream client only exposed raw HTML fetch + archive download. This
//! adapter targets ASSRT's public JSON API (`https://api.assrt.net/v1`), which
//! is the documented integration surface and returns structured search /
//! detail payloads suitable for caching as raw JSON.
//!
//! Endpoints:
//! - `GET /v1/sub/search?token=&q=&cnt=&pos=` — full-text search
//! - `GET /v1/sub/detail?token=&id=` — per-sub detail (download URLs, etc.)

use serde_json::Value;
use sha2::{Digest, Sha256};
use tokimo_core::{CoreError, CoreResult};

pub const ASSRT_API_BASE: &str = "https://api.assrt.net/v1";
pub const ASSRT_USER_AGENT: &str = "tokimo-server/0.1 (https://github.com/tokimo-lab/tokimo-server)";

/// Stable cache key for a search request: sha256 of `{q}|{cnt}|{pos}`.
pub fn search_cache_key(q: &str, cnt: Option<u32>, pos: Option<u32>) -> String {
    let raw = format!(
        "{}|{}|{}",
        q.trim(),
        cnt.map(|v| v.to_string()).unwrap_or_default(),
        pos.map(|v| v.to_string()).unwrap_or_default(),
    );
    let mut h = Sha256::new();
    h.update(raw.as_bytes());
    format!("{:x}", h.finalize())
}

async fn assrt_get(http: &reqwest::Client, url: &str) -> CoreResult<Value> {
    let resp = http
        .get(url)
        .header(reqwest::header::USER_AGENT, ASSRT_USER_AGENT)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(CoreError::Upstream)?;
    if !resp.status().is_success() {
        return Err(CoreError::Provider(format!("ASSRT returned status {}", resp.status())));
    }
    resp.json::<Value>().await.map_err(CoreError::Upstream)
}

/// `GET /v1/sub/search` — returns raw JSON.
pub async fn search(
    http: &reqwest::Client,
    token: &str,
    q: &str,
    cnt: Option<u32>,
    pos: Option<u32>,
) -> CoreResult<Value> {
    let mut url = format!(
        "{}/sub/search?token={}&q={}",
        ASSRT_API_BASE,
        urlencoding::encode(token),
        urlencoding::encode(q),
    );
    if let Some(c) = cnt {
        url.push_str(&format!("&cnt={}", c));
    }
    if let Some(p) = pos {
        url.push_str(&format!("&pos={}", p));
    }
    assrt_get(http, &url).await
}

/// `GET /v1/sub/detail?id={sub_id}` — returns raw JSON.
pub async fn fetch_detail(http: &reqwest::Client, token: &str, sub_id: &str) -> CoreResult<Value> {
    let url = format!(
        "{}/sub/detail?token={}&id={}",
        ASSRT_API_BASE,
        urlencoding::encode(token),
        urlencoding::encode(sub_id),
    );
    assrt_get(http, &url).await
}

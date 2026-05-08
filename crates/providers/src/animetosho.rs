//! AnimeTosho feed adapter — anime release / subtitle attachment metadata.
//!
//! Two endpoints are proxied:
//!
//! - `GET https://feed.animetosho.org/json?q={query}` — entry search.
//! - `GET https://feed.animetosho.org/json?show=torrent&id={id}` — torrent
//!   details, including the list of files and their subtitle attachments.
//!
//! Both endpoints are public and require no API key.
//!
//! AnimeTosho stores subtitle attachments as xz-compressed blobs at
//! `https://animetosho.org/storage/attach/{hex8(id)}/{id}.xz`. The download
//! step is intentionally NOT proxied here — clients can hit that URL
//! directly.

use tokimo_core::{CoreError, CoreResult};

const FEED_API_URL: &str = "https://feed.animetosho.org/json";

/// Cache key for the entry-list search endpoint.
pub fn search_cache_key(query: &str) -> String {
    format!("animetosho:search:{}", query.trim().to_lowercase())
}

/// Cache key for the per-torrent detail endpoint.
pub fn torrent_cache_key(id: u64) -> String {
    format!("animetosho:torrent:{id}")
}

/// Search AnimeTosho for entries matching the given query.
pub async fn search(http: &reqwest::Client, query: &str) -> CoreResult<serde_json::Value> {
    let resp = http
        .get(FEED_API_URL)
        .query(&[("q", query)])
        .send()
        .await
        .map_err(CoreError::Upstream)?;

    let status = resp.status();
    if !status.is_success() {
        return Err(CoreError::Provider(format!("animetosho returned status {status}")));
    }
    resp.json::<serde_json::Value>().await.map_err(CoreError::Upstream)
}

/// Fetch torrent details (file list + attachments) for a single entry.
pub async fn torrent(http: &reqwest::Client, id: u64) -> CoreResult<serde_json::Value> {
    let id_str = id.to_string();
    let resp = http
        .get(FEED_API_URL)
        .query(&[("show", "torrent"), ("id", id_str.as_str())])
        .send()
        .await
        .map_err(CoreError::Upstream)?;

    let status = resp.status();
    if !status.is_success() {
        return Err(CoreError::Provider(format!("animetosho returned status {status}")));
    }
    resp.json::<serde_json::Value>().await.map_err(CoreError::Upstream)
}

//! iTunes Search API adapter.
//!
//! Endpoint: `https://itunes.apple.com/search` — album cover lookup with
//! 3000×3000 high-resolution fallback.

use tokimo_core::{CoreError, CoreResult};

pub const ITUNES_SEARCH_URL: &str = "https://itunes.apple.com/search";

/// Search for album cover art.
pub async fn search_album_cover(http: &reqwest::Client, artist: &str, album: &str) -> CoreResult<serde_json::Value> {
    let query = format!("{} {}", artist, album);
    let encoded_query = urlencoding::encode(&query);
    let url = format!(
        "{}?term={}&entity=album&country=cn&limit=5",
        ITUNES_SEARCH_URL, encoded_query
    );

    let resp = http.get(&url).send().await.map_err(CoreError::Upstream)?;
    let status = resp.status();
    if !status.is_success() {
        return Err(CoreError::Provider(format!("itunes returned status {status}")));
    }
    resp.json::<serde_json::Value>().await.map_err(CoreError::Upstream)
}

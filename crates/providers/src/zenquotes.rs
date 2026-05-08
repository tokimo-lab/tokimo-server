//! ZenQuotes adapter.
//!
//! Endpoint: `https://zenquotes.io/api/random` — returns a JSON array
//! `[{q, a, h}]`. Aggressive rate limiting upstream, so we cache for 30
//! minutes with a single shared cache row across all callers.

use tokimo_core::{CoreError, CoreResult};

pub const ZENQUOTES_RANDOM_URL: &str = "https://zenquotes.io/api/random";

/// Fetch a random quote (raw JSON array).
pub async fn fetch_random(http: &reqwest::Client) -> CoreResult<serde_json::Value> {
    let resp = http
        .get(ZENQUOTES_RANDOM_URL)
        .send()
        .await
        .map_err(CoreError::Upstream)?;
    let status = resp.status();
    if !status.is_success() {
        return Err(CoreError::Provider(format!("zenquotes returned status {status}")));
    }
    resp.json::<serde_json::Value>().await.map_err(CoreError::Upstream)
}

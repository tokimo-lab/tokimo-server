//! RegIeLive (Romanian subtitles) search adapter.
//!
//! Endpoint: `GET https://api.regielive.ro/bazarr/search.php?nume={query}`
//! Requires a fixed `RL-API` header value baked in (per upstream Bazarr
//! convention). No per-user / per-server API key.

use tokimo_core::{CoreError, CoreResult};

const REGIELIVE_API_URL: &str = "https://api.regielive.ro/bazarr/search.php";
const RL_API_KEY: &str = "API-BAZARR-YTZ-SL";
const REGIELIVE_USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:72.0) Gecko/20100101 Firefox/72.0";

/// Build the cache key from the query (lowercased, trimmed).
pub fn cache_key(nume: &str) -> String {
    format!("regielive:{}", nume.trim().to_lowercase())
}

/// Search RegIeLive for the given title.
pub async fn search(http: &reqwest::Client, nume: &str) -> CoreResult<serde_json::Value> {
    let url = format!("{REGIELIVE_API_URL}?nume={}", urlencoding::encode(nume.trim()));

    let resp = http
        .get(&url)
        .header("RL-API", RL_API_KEY)
        .header(reqwest::header::USER_AGENT, REGIELIVE_USER_AGENT)
        .send()
        .await
        .map_err(CoreError::Upstream)?;

    let status = resp.status();
    if !status.is_success() {
        return Err(CoreError::Provider(format!("regielive returned status {status}")));
    }
    resp.json::<serde_json::Value>().await.map_err(CoreError::Upstream)
}

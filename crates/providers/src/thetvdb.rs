//! TheTVDB v4 adapter — copied + adapted from
//! tokimo/packages/rust-client-api/src/metadata_providers/thetvdb.rs
//!
//! Differences from the upstream client:
//! - In-memory `RequestCache` removed (handled by tokimo-server's DB layer)
//! - Token state externalized: callers pass an existing valid token in or
//!   call `login` themselves; this module exposes pure async fns. The
//!   route layer is responsible for persisting the token in
//!   `thetvdb_token_cache` and refreshing on expiry.

use serde::{Deserialize, Serialize};
use tokimo_core::{CoreError, CoreResult};

pub const THETVDB_BASE: &str = "https://api4.thetvdb.com/v4";
/// Tokens are issued for ~30 days; we refresh slightly earlier to be safe.
pub const TOKEN_TTL_SECONDS: i64 = 24 * 60 * 60;

#[derive(Serialize)]
struct LoginBody<'a> {
    apikey: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pin: Option<&'a str>,
}

#[derive(Deserialize)]
struct LoginResponse {
    status: Option<String>,
    data: Option<LoginData>,
}

#[derive(Deserialize)]
struct LoginData {
    token: Option<String>,
}

/// POST /login → token. Returns the JWT string on success.
pub async fn login(http: &reqwest::Client, api_key: &str, pin: Option<&str>) -> CoreResult<String> {
    let body = LoginBody { apikey: api_key, pin };
    let resp = http
        .post(format!("{}/login", THETVDB_BASE))
        .json(&body)
        .send()
        .await
        .map_err(CoreError::Upstream)?;
    if !resp.status().is_success() {
        return Err(CoreError::Provider(format!("TheTVDB login failed: {}", resp.status())));
    }
    let data: LoginResponse = resp.json().await.map_err(CoreError::Upstream)?;
    if data.status.as_deref() != Some("success") {
        return Err(CoreError::Provider("TheTVDB login: invalid response".into()));
    }
    data.data
        .and_then(|d| d.token)
        .ok_or_else(|| CoreError::Provider("TheTVDB login: no token".into()))
}

async fn authed_get(http: &reqwest::Client, token: &str, url: &str) -> CoreResult<serde_json::Value> {
    let resp = http
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(CoreError::Upstream)?;
    if !resp.status().is_success() {
        return Err(CoreError::Provider(format!("TheTVDB API error: {}", resp.status())));
    }
    resp.json::<serde_json::Value>().await.map_err(CoreError::Upstream)
}

/// GET /series/:id/extended
pub async fn fetch_series(http: &reqwest::Client, token: &str, id: i64) -> CoreResult<serde_json::Value> {
    let url = format!("{}/series/{}/extended", THETVDB_BASE, id);
    authed_get(http, token, &url).await
}

/// GET /series/:id/episodes/default — full episodes listing for the series.
pub async fn fetch_series_episodes(http: &reqwest::Client, token: &str, id: i64) -> CoreResult<serde_json::Value> {
    let url = format!("{}/series/{}/episodes/default", THETVDB_BASE, id);
    authed_get(http, token, &url).await
}

/// GET /episodes/:id/extended — single-episode detail.
pub async fn fetch_episode(http: &reqwest::Client, token: &str, id: i64) -> CoreResult<serde_json::Value> {
    let url = format!("{}/episodes/{}/extended", THETVDB_BASE, id);
    authed_get(http, token, &url).await
}

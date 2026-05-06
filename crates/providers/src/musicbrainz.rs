//! MusicBrainz adapter — copied + adapted from
//! tokimo/packages/rust-client-api/src/metadata_providers/musicbrainz.rs
//!
//! Differences from the upstream client:
//! - In-memory rate limiter removed; tokimo-server's PgRateLimiter
//!   ("musicbrainz" bucket, seeded to 1 req/sec) enforces the TOS limit
//!   across the whole process — and across processes via the shared row.
//! - The upstream `User-Agent` constant is replaced with a caller-supplied
//!   string sourced from `MUSICBRAINZ_USER_AGENT` env (with a sensible
//!   fallback), since MusicBrainz's TOS requires identifying the app.

use tokimo_core::{CoreError, CoreResult};

pub const MB_BASE_URL: &str = "https://musicbrainz.org/ws/2";

async fn mb_get(http: &reqwest::Client, user_agent: &str, url: &str) -> CoreResult<serde_json::Value> {
    let resp = http
        .get(url)
        .header("Accept", "application/json")
        .header("User-Agent", user_agent)
        .send()
        .await
        .map_err(CoreError::Upstream)?;
    if !resp.status().is_success() {
        return Err(CoreError::Provider(format!(
            "MusicBrainz {} returned status {}",
            url,
            resp.status()
        )));
    }
    resp.json::<serde_json::Value>().await.map_err(CoreError::Upstream)
}

/// GET /artist/{mbid}?fmt=json&inc=aliases+url-rels
pub async fn fetch_artist(http: &reqwest::Client, user_agent: &str, mbid: &str) -> CoreResult<serde_json::Value> {
    let url = format!(
        "{}/artist/{}?fmt=json&inc=aliases+url-rels",
        MB_BASE_URL,
        urlencoding::encode(mbid)
    );
    mb_get(http, user_agent, &url).await
}

/// GET /release/{mbid}?fmt=json&inc=artists+labels+recordings+release-groups+media
pub async fn fetch_release(http: &reqwest::Client, user_agent: &str, mbid: &str) -> CoreResult<serde_json::Value> {
    let url = format!(
        "{}/release/{}?fmt=json&inc=artists+labels+recordings+release-groups+media",
        MB_BASE_URL,
        urlencoding::encode(mbid)
    );
    mb_get(http, user_agent, &url).await
}

/// GET /recording/{mbid}?fmt=json&inc=artists+releases+isrcs
pub async fn fetch_recording(http: &reqwest::Client, user_agent: &str, mbid: &str) -> CoreResult<serde_json::Value> {
    let url = format!(
        "{}/recording/{}?fmt=json&inc=artists+releases+isrcs",
        MB_BASE_URL,
        urlencoding::encode(mbid)
    );
    mb_get(http, user_agent, &url).await
}

/// GET /{search_type}?query={q}&fmt=json
///
/// `search_type` must be one of MusicBrainz's entity types:
/// `artist`, `release`, `recording`, `release-group`, `label`, `work`.
pub async fn search(
    http: &reqwest::Client,
    user_agent: &str,
    search_type: &str,
    q: &str,
) -> CoreResult<serde_json::Value> {
    let url = format!(
        "{}/{}?query={}&fmt=json",
        MB_BASE_URL,
        urlencoding::encode(search_type),
        urlencoding::encode(q)
    );
    mb_get(http, user_agent, &url).await
}

//! LRCLIB lyrics adapter — copied + adapted from
//! tokimo/packages/rust-client-api/src/metadata_providers/lrclib.rs
//!
//! Differences from the upstream client:
//! - Returns raw JSON instead of the upstream `LyricsResult` DTO;
//!   tokimo-server's DB cache stores the unparsed body.
//! - The two-step strategy (`/api/get` exact → `/api/search` lenient) is
//!   preserved; on `/api/get` 404, we fall back to `/api/search` and pick
//!   the first hit. A truly empty result is returned as
//!   `CoreError::NotFound` so the route handler can map it to 404.

use serde_json::Value;
use sha2::{Digest, Sha256};
use tokimo_core::{CoreError, CoreResult};

pub const LRCLIB_BASE_URL: &str = "https://lrclib.net/api";
const USER_AGENT: &str = "tokimo-server/0.1 (https://github.com/tokimo-lab/tokimo-server)";

/// Build a stable cache key from `(artist, track, album?, duration?)`.
///
/// Mirrors the upstream key shape: `format!("{artist}|{track}|{album}|{duration}")`
/// hashed with SHA-256 and hex-encoded.
pub fn cache_key(artist: &str, track: &str, album: Option<&str>, duration: Option<u32>) -> String {
    let raw = format!(
        "{}|{}|{}|{}",
        artist,
        track,
        album.unwrap_or(""),
        duration.map(|d| d.to_string()).unwrap_or_default()
    );
    let mut h = Sha256::new();
    h.update(raw.as_bytes());
    hex::encode(h.finalize())
}

async fn lrclib_get(http: &reqwest::Client, url: &str) -> CoreResult<reqwest::Response> {
    http.get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .map_err(CoreError::Upstream)
}

/// Fetch a single best-match lyrics object for `(artist, track, album?, duration?)`.
///
/// Tries `/api/get` first (exact match); on 404 falls back to `/api/search`
/// and returns the first result. Returns `CoreError::NotFound` when no
/// match is found.
pub async fn fetch_lyrics(
    http: &reqwest::Client,
    artist: &str,
    track: &str,
    album: Option<&str>,
    duration: Option<u32>,
) -> CoreResult<Value> {
    // Step 1: /api/get with full params.
    let mut get_url = format!(
        "{}/get?artist_name={}&track_name={}",
        LRCLIB_BASE_URL,
        urlencoding::encode(artist),
        urlencoding::encode(track),
    );
    if let Some(album) = album {
        get_url.push_str(&format!("&album_name={}", urlencoding::encode(album)));
    }
    if let Some(dur) = duration {
        get_url.push_str(&format!("&duration={}", dur));
    }

    let resp = lrclib_get(http, &get_url).await?;

    if resp.status().is_success() {
        return resp.json::<Value>().await.map_err(CoreError::Upstream);
    }

    if resp.status().as_u16() != 404 {
        return Err(CoreError::Provider(format!(
            "LRCLIB /get returned status {}",
            resp.status()
        )));
    }

    // Step 2: fallback /api/search (no duration constraint).
    let search_url = format!(
        "{}/search?artist_name={}&track_name={}",
        LRCLIB_BASE_URL,
        urlencoding::encode(artist),
        urlencoding::encode(track),
    );
    let search_resp = lrclib_get(http, &search_url).await?;
    if !search_resp.status().is_success() {
        return Err(CoreError::NotFound);
    }
    let arr: Vec<Value> = search_resp.json().await.map_err(CoreError::Upstream)?;
    arr.into_iter().next().ok_or(CoreError::NotFound)
}

/// GET /api/search?q={q} — full-text search; returns the raw JSON array.
pub async fn search(http: &reqwest::Client, q: &str) -> CoreResult<Value> {
    let url = format!("{}/search?q={}", LRCLIB_BASE_URL, urlencoding::encode(q));
    let resp = lrclib_get(http, &url).await?;
    if !resp.status().is_success() {
        return Err(CoreError::Provider(format!(
            "LRCLIB /search returned status {}",
            resp.status()
        )));
    }
    resp.json::<Value>().await.map_err(CoreError::Upstream)
}

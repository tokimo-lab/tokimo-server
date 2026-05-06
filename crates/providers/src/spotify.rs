//! Spotify Web API adapter — copied + adapted from
//! tokimo/packages/rust-client-api/src/metadata_providers/spotify.rs
//!
//! Differences from the upstream client:
//! - In-memory `RequestCache` and `RwLock<TokenState>` removed; the route
//!   layer in tokimo-server persists the bearer token in
//!   `spotify_token_cache` and refreshes on expiry (mirrors the TheTVDB
//!   pattern).
//! - This module exposes pure async fns returning raw JSON (no DTO mapping).

use base64::Engine;
use serde::Deserialize;
use tokimo_core::{CoreError, CoreResult};

pub const SPOTIFY_TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
pub const SPOTIFY_API_BASE: &str = "https://api.spotify.com/v1";

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
}

/// Result of a successful client-credentials token exchange.
pub struct SpotifyToken {
    pub access_token: String,
    /// Lifetime of the token in seconds, as reported by Spotify.
    pub expires_in: i64,
}

/// POST `https://accounts.spotify.com/api/token` (client-credentials grant).
pub async fn request_token(http: &reqwest::Client, client_id: &str, client_secret: &str) -> CoreResult<SpotifyToken> {
    let credentials = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", client_id, client_secret));

    let resp = http
        .post(SPOTIFY_TOKEN_URL)
        .header("Authorization", format!("Basic {}", credentials))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("grant_type=client_credentials")
        .send()
        .await
        .map_err(CoreError::Upstream)?;

    if !resp.status().is_success() {
        return Err(CoreError::Provider(format!("Spotify token error: {}", resp.status())));
    }

    let data: TokenResponse = resp.json().await.map_err(CoreError::Upstream)?;
    Ok(SpotifyToken {
        access_token: data.access_token,
        expires_in: data.expires_in,
    })
}

async fn authed_get(http: &reqwest::Client, token: &str, url: &str) -> CoreResult<serde_json::Value> {
    let resp = http
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(CoreError::Upstream)?;
    if !resp.status().is_success() {
        return Err(CoreError::Provider(format!(
            "Spotify API {} returned status {}",
            url,
            resp.status()
        )));
    }
    resp.json::<serde_json::Value>().await.map_err(CoreError::Upstream)
}

/// GET /v1/artists/{id}
pub async fn fetch_artist(http: &reqwest::Client, token: &str, id: &str) -> CoreResult<serde_json::Value> {
    let url = format!("{}/artists/{}", SPOTIFY_API_BASE, urlencoding::encode(id));
    authed_get(http, token, &url).await
}

/// GET /v1/albums/{id}
pub async fn fetch_album(http: &reqwest::Client, token: &str, id: &str) -> CoreResult<serde_json::Value> {
    let url = format!("{}/albums/{}", SPOTIFY_API_BASE, urlencoding::encode(id));
    authed_get(http, token, &url).await
}

/// GET /v1/tracks/{id}
pub async fn fetch_track(http: &reqwest::Client, token: &str, id: &str) -> CoreResult<serde_json::Value> {
    let url = format!("{}/tracks/{}", SPOTIFY_API_BASE, urlencoding::encode(id));
    authed_get(http, token, &url).await
}

/// GET /v1/search?q={q}&type={type}
///
/// `search_type` is the comma-separated Spotify object type filter
/// (`album`, `artist`, `playlist`, `track`, `show`, `episode`, `audiobook`).
pub async fn search(http: &reqwest::Client, token: &str, q: &str, search_type: &str) -> CoreResult<serde_json::Value> {
    let url = format!(
        "{}/search?q={}&type={}",
        SPOTIFY_API_BASE,
        urlencoding::encode(q),
        urlencoding::encode(search_type)
    );
    authed_get(http, token, &url).await
}

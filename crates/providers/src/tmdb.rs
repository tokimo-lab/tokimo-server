use serde::{Deserialize, Serialize};
use tokimo_core::{CoreError, CoreResult, Storage};

use crate::common::download_to_storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbMovie {
    pub id: i32,
    pub title: String,
    pub original_title: Option<String>,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub runtime: Option<i32>,
    pub vote_average: Option<f64>,
    pub vote_count: Option<i32>,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TmdbMovieResponse {
    pub id: i32,
    pub title: String,
    pub original_title: Option<String>,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub runtime: Option<i32>,
    pub vote_average: Option<f64>,
    pub vote_count: Option<i32>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
}

pub async fn fetch_movie(
    http: &reqwest::Client,
    api_key: &str,
    movie_id: i32,
) -> CoreResult<(TmdbMovieResponse, serde_json::Value)> {
    let url = format!("https://api.themoviedb.org/3/movie/{}?api_key={}", movie_id, api_key);

    let response = http.get(&url).send().await.map_err(CoreError::Upstream)?;

    if !response.status().is_success() {
        return Err(CoreError::Provider(format!("TMDB API error: {}", response.status())));
    }

    let raw_json: serde_json::Value = response.json().await.map_err(CoreError::Upstream)?;

    let movie: TmdbMovieResponse = serde_json::from_value(raw_json.clone())
        .map_err(|e| CoreError::Provider(format!("Failed to parse TMDB response: {}", e)))?;

    Ok((movie, raw_json))
}

/// TMDB API base URL.
pub const TMDB_API_BASE: &str = "https://api.themoviedb.org/3";

/// Generic TMDB GET that returns the raw JSON.
///
/// `path` should be a leading-slash absolute path under `/3/...`, e.g.
/// `/tv/1399`, `/tv/1399/season/1`, `/person/287`. `extra_query` lets the
/// caller append things like `append_to_response=credits` on top of the
/// always-injected `api_key` and `language` params.
pub async fn fetch_tmdb_path(
    http: &reqwest::Client,
    api_key: &str,
    path: &str,
    extra_query: &[(&str, &str)],
) -> CoreResult<serde_json::Value> {
    let mut url = format!(
        "{}{}?api_key={}&language=zh-CN",
        TMDB_API_BASE,
        path,
        urlencoding::encode(api_key)
    );
    for (k, v) in extra_query {
        url.push('&');
        url.push_str(&urlencoding::encode(k));
        url.push('=');
        url.push_str(&urlencoding::encode(v));
    }
    crate::common::http_get_json(http, &url).await
}

pub async fn fetch_tv(http: &reqwest::Client, api_key: &str, tv_id: i32) -> CoreResult<serde_json::Value> {
    fetch_tmdb_path(
        http,
        api_key,
        &format!("/tv/{}", tv_id),
        &[("append_to_response", "external_ids,credits")],
    )
    .await
}

pub async fn fetch_tv_season(
    http: &reqwest::Client,
    api_key: &str,
    tv_id: i32,
    season: i32,
) -> CoreResult<serde_json::Value> {
    fetch_tmdb_path(
        http,
        api_key,
        &format!("/tv/{}/season/{}", tv_id, season),
        &[("append_to_response", "credits")],
    )
    .await
}

pub async fn fetch_tv_episode(
    http: &reqwest::Client,
    api_key: &str,
    tv_id: i32,
    season: i32,
    episode: i32,
) -> CoreResult<serde_json::Value> {
    fetch_tmdb_path(
        http,
        api_key,
        &format!("/tv/{}/season/{}/episode/{}", tv_id, season, episode),
        &[("append_to_response", "credits")],
    )
    .await
}

pub async fn fetch_person(http: &reqwest::Client, api_key: &str, person_id: i32) -> CoreResult<serde_json::Value> {
    fetch_tmdb_path(http, api_key, &format!("/person/{}", person_id), &[]).await
}

/// TMDB image base URL.
pub const TMDB_IMAGE_BASE: &str = "https://image.tmdb.org/t/p/original";

/// Download a TMDB image (given a relative path like `/abc.jpg`) to storage.
///
/// Thin wrapper around [`download_to_storage`] that prepends the TMDB CDN.
pub async fn download_image(
    http: &reqwest::Client,
    image_path: &str,
    storage: &dyn Storage,
) -> CoreResult<(String, String)> {
    let full_url = format!("{}{}", TMDB_IMAGE_BASE, image_path);
    download_to_storage(http, &full_url, storage, "tmdb").await
}

#[cfg(all(test, feature = "live-api"))]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_tmdb_fetch_movie() {
        let api_key = std::env::var("TMDB_API_KEY").expect("TMDB_API_KEY not set");
        let client = reqwest::Client::new();
        let result = fetch_movie(&client, &api_key, 550).await;
        assert!(result.is_ok());
    }
}

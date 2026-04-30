use bytes::Bytes;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokimo_core::{compute_storage_key, CoreError, CoreResult, Storage};

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

pub async fn download_image(
    http: &reqwest::Client,
    image_path: &str,
    storage: &dyn Storage,
) -> CoreResult<(String, String)> {
    let full_url = format!("https://image.tmdb.org/t/p/original{}", image_path);

    let response = http.get(&full_url).send().await.map_err(CoreError::Upstream)?;

    if !response.status().is_success() {
        return Err(CoreError::Provider(format!(
            "Failed to download image: {}",
            response.status()
        )));
    }

    let bytes = response.bytes().await.map_err(CoreError::Upstream)?;

    let mime = infer::get(&bytes)
        .map(|t| t.mime_type().to_string())
        .unwrap_or_else(|| "image/jpeg".to_string());

    let ext = mime.split('/').nth(1).unwrap_or("jpg");

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let sha256_hex = hex::encode(hasher.finalize());

    let storage_key = compute_storage_key("tmdb", &sha256_hex, ext);

    storage.put(&storage_key, Bytes::from(bytes.to_vec()), &mime).await?;

    Ok((sha256_hex, storage_key))
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

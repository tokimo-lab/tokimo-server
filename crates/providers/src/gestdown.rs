//! Gestdown (Addic7ed mirror) adapter.
//!
//! Endpoints:
//! - `GET https://api.gestdown.info/shows/search/{title}`
//! - `GET https://api.gestdown.info/subtitles/get/{show_id}/{season}/{episode}/{lang}`

use tokimo_core::{CoreError, CoreResult};

const GESTDOWN_BASE_URL: &str = "https://api.gestdown.info";
const GESTDOWN_USER_AGENT: &str = "Bazarr";

pub fn shows_cache_key(title: &str) -> String {
    format!("gestdown:shows:{}", title.trim().to_lowercase())
}

pub fn subs_cache_key(show_id: &str, season: u32, episode: u32, lang: &str) -> String {
    format!("gestdown:subs:{show_id}:{season}:{episode}:{lang}")
}

async fn get_json(http: &reqwest::Client, url: &str) -> CoreResult<serde_json::Value> {
    let resp = http
        .get(url)
        .header(reqwest::header::USER_AGENT, GESTDOWN_USER_AGENT)
        .send()
        .await
        .map_err(CoreError::Upstream)?;
    let status = resp.status();
    if status.as_u16() == 404 {
        return Err(CoreError::NotFound);
    }
    if !status.is_success() {
        return Err(CoreError::Provider(format!("gestdown returned status {status}")));
    }
    resp.json::<serde_json::Value>().await.map_err(CoreError::Upstream)
}

/// `GET /shows/search/{title}`
pub async fn search_shows(http: &reqwest::Client, title: &str) -> CoreResult<serde_json::Value> {
    let url = format!("{GESTDOWN_BASE_URL}/shows/search/{}", urlencoding::encode(title.trim()));
    get_json(http, &url).await
}

/// `GET /subtitles/get/{show_id}/{season}/{episode}/{lang}`
pub async fn get_subtitles(
    http: &reqwest::Client,
    show_id: &str,
    season: u32,
    episode: u32,
    lang: &str,
) -> CoreResult<serde_json::Value> {
    let url = format!(
        "{GESTDOWN_BASE_URL}/subtitles/get/{}/{}/{}/{}",
        urlencoding::encode(show_id),
        season,
        episode,
        urlencoding::encode(lang)
    );
    get_json(http, &url).await
}

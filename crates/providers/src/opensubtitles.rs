//! OpenSubtitles search adapter.
//!
//! Endpoint: `GET https://api.opensubtitles.com/api/v1/subtitles?{params}`
//! Requires an `Api-Key` header (per-server, from env) and a `User-Agent`.

use sha2::{Digest, Sha256};
use tokimo_core::{CoreError, CoreResult};

const OS_API_BASE: &str = "https://api.opensubtitles.com/api/v1";
const OS_USER_AGENT: &str = "tokimo-server/0.1";

/// Stable cache key from the (query, imdb, tmdb, languages) tuple, hashed.
pub fn cache_key(query: &str, imdb: &str, tmdb: &str, langs: &str) -> String {
    let raw = format!("{query}|{imdb}|{tmdb}|{langs}");
    let mut h = Sha256::new();
    h.update(raw.as_bytes());
    format!("opensubtitles:{}", hex::encode(h.finalize()))
}

/// Strip a leading `tt` prefix from an IMDB id.
pub fn normalize_imdb(imdb: &str) -> &str {
    imdb.strip_prefix("tt").unwrap_or(imdb)
}

/// Search subtitles. At least one of `query` / `imdb_id` must be provided.
pub async fn search(
    http: &reqwest::Client,
    api_key: &str,
    query: Option<&str>,
    imdb_id: Option<&str>,
    tmdb_id: Option<&str>,
    languages: Option<&str>,
) -> CoreResult<serde_json::Value> {
    let mut params: Vec<(&str, String)> = Vec::new();

    if let Some(q) = query.map(str::trim).filter(|s| !s.is_empty()) {
        params.push(("query", q.to_string()));
    }
    if let Some(id) = imdb_id.map(str::trim).filter(|s| !s.is_empty()) {
        params.push(("imdb_id", normalize_imdb(id).to_string()));
    }
    if let Some(id) = tmdb_id.map(str::trim).filter(|s| !s.is_empty()) {
        params.push(("tmdb_id", id.to_string()));
    }
    if let Some(langs) = languages.map(str::trim).filter(|s| !s.is_empty()) {
        params.push(("languages", langs.to_string()));
    }

    if params.is_empty() {
        return Err(CoreError::Provider(
            "opensubtitles: at least one of query / imdb_id is required".into(),
        ));
    }

    let url = format!("{OS_API_BASE}/subtitles");
    let resp = http
        .get(&url)
        .header("Api-Key", api_key)
        .header(reqwest::header::USER_AGENT, OS_USER_AGENT)
        .query(&params)
        .send()
        .await
        .map_err(CoreError::Upstream)?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(CoreError::Provider(format!(
            "opensubtitles returned status {status}: {body}"
        )));
    }
    resp.json::<serde_json::Value>().await.map_err(CoreError::Upstream)
}

//! GitHub Releases adapter — adapted from
//! tokimo/packages/rust-client-api/src/github_releases.rs
//!
//! The upstream client only parsed `tag_name`; here we proxy the full
//! `https://api.github.com/repos/{owner}/{repo}/releases/...` payload as
//! raw JSON so callers can read assets, body, published_at, etc.
//!
//! Endpoints:
//! - `GET /repos/{owner}/{repo}/releases/latest` — latest release
//! - `GET /repos/{owner}/{repo}/releases` — list releases

use serde_json::Value;
use tokimo_core::{CoreError, CoreResult};

pub const GITHUB_API_BASE: &str = "https://api.github.com";
pub const GITHUB_USER_AGENT: &str = "tokimo-server/0.1 (https://github.com/tokimo-lab/tokimo-server)";

async fn github_get(http: &reqwest::Client, url: &str, token: Option<&str>) -> CoreResult<Value> {
    let mut req = http
        .get(url)
        .header(reqwest::header::USER_AGENT, GITHUB_USER_AGENT)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28");
    if let Some(t) = token {
        req = req.header(reqwest::header::AUTHORIZATION, format!("Bearer {}", t));
    }
    let resp = req.send().await.map_err(CoreError::Upstream)?;
    if !resp.status().is_success() {
        return Err(CoreError::Provider(format!(
            "GitHub API returned status {}",
            resp.status()
        )));
    }
    resp.json::<Value>().await.map_err(CoreError::Upstream)
}

/// `GET /repos/{owner}/{repo}/releases/latest` — returns raw JSON.
pub async fn fetch_latest_release(
    http: &reqwest::Client,
    owner: &str,
    repo: &str,
    token: Option<&str>,
) -> CoreResult<Value> {
    let url = format!("{}/repos/{}/{}/releases/latest", GITHUB_API_BASE, owner, repo);
    github_get(http, &url, token).await
}

/// `GET /repos/{owner}/{repo}/releases` — returns raw JSON array.
pub async fn list_releases(
    http: &reqwest::Client,
    owner: &str,
    repo: &str,
    per_page: Option<u32>,
    page: Option<u32>,
    token: Option<&str>,
) -> CoreResult<Value> {
    let mut url = format!("{}/repos/{}/{}/releases", GITHUB_API_BASE, owner, repo);
    let mut query: Vec<String> = Vec::new();
    if let Some(pp) = per_page {
        query.push(format!("per_page={}", pp));
    }
    if let Some(p) = page {
        query.push(format!("page={}", p));
    }
    if !query.is_empty() {
        url.push('?');
        url.push_str(&query.join("&"));
    }
    github_get(http, &url, token).await
}

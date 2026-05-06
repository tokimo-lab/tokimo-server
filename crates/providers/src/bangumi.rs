//! Bangumi (bgm.tv) adapter — copied + adapted from
//! tokimo/packages/rust-client-api/src/metadata_providers/bangumi.rs
//!
//! Returns raw JSON; persistence + single-flight handled at the route layer.
//! Bangumi requires a non-default `User-Agent` per their API guidelines, so
//! the route layer must supply one from `AppConfig::bangumi_user_agent`.

use serde_json::Value;
use tokimo_core::{CoreError, CoreResult};

pub const BANGUMI_BASE: &str = "https://api.bgm.tv/v0";
pub const BANGUMI_CALENDAR_URL: &str = "https://api.bgm.tv/calendar";

fn headers(user_agent: &str, access_token: Option<&str>) -> reqwest::header::HeaderMap {
    let mut h = reqwest::header::HeaderMap::new();
    h.insert("User-Agent", user_agent.parse().unwrap());
    h.insert("Content-Type", "application/json".parse().unwrap());
    if let Some(t) = access_token {
        h.insert("Authorization", format!("Bearer {t}").parse().unwrap());
    }
    h
}

async fn read_json(resp: reqwest::Response) -> CoreResult<Value> {
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(CoreError::Provider(format!("Bangumi API {status}: {body}")));
    }
    resp.json::<Value>().await.map_err(CoreError::Upstream)
}

/// Fetch subject detail by id; returns Ok(None) on 404.
pub async fn fetch_subject(
    http: &reqwest::Client,
    user_agent: &str,
    access_token: Option<&str>,
    id: i64,
) -> CoreResult<Option<Value>> {
    let url = format!("{}/subjects/{}", BANGUMI_BASE, id);
    let resp = http
        .get(&url)
        .headers(headers(user_agent, access_token))
        .send()
        .await
        .map_err(CoreError::Upstream)?;
    if resp.status().as_u16() == 404 {
        return Ok(None);
    }
    let raw = read_json(resp).await?;
    Ok(Some(raw))
}

/// Search subjects via `GET /v0/search/subjects`.
pub async fn search(
    http: &reqwest::Client,
    user_agent: &str,
    access_token: Option<&str>,
    keyword: &str,
    subject_type: u8,
    limit: u32,
) -> CoreResult<Value> {
    let url = format!(
        "{}/search/subjects?keyword={}&type={}&limit={}",
        BANGUMI_BASE,
        urlencoding::encode(keyword),
        subject_type,
        limit
    );
    let resp = http
        .get(&url)
        .headers(headers(user_agent, access_token))
        .send()
        .await
        .map_err(CoreError::Upstream)?;
    read_json(resp).await
}

/// Browse subjects via `POST /v0/search/subjects` (sorted by heat).
pub async fn browse(
    http: &reqwest::Client,
    user_agent: &str,
    access_token: Option<&str>,
    subject_type: u8,
    limit: u32,
) -> CoreResult<Value> {
    let body = serde_json::json!({
        "keyword": "",
        "filter": { "type": [subject_type], "nsfw": false },
        "sort": "heat",
        "limit": limit,
        "offset": 0,
    });
    let resp = http
        .post(format!("{}/search/subjects", BANGUMI_BASE))
        .headers(headers(user_agent, access_token))
        .json(&body)
        .send()
        .await
        .map_err(CoreError::Upstream)?;
    read_json(resp).await
}

/// Weekly anime calendar at `https://api.bgm.tv/calendar` (outside `/v0`).
pub async fn fetch_calendar(http: &reqwest::Client, user_agent: &str, access_token: Option<&str>) -> CoreResult<Value> {
    let resp = http
        .get(BANGUMI_CALENDAR_URL)
        .headers(headers(user_agent, access_token))
        .send()
        .await
        .map_err(CoreError::Upstream)?;
    read_json(resp).await
}

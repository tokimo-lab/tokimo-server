//! Wikipedia adapter — adapted from
//! tokimo/packages/rust-client-api/src/metadata_providers/wikipedia.rs
//!
//! The upstream module specialised on "fetch zh-language extract for an
//! artist biography URL". Here we expose the underlying REST summary
//! endpoint generically, parameterised by `lang` + `title`, so callers
//! can request any language. The biography helper can be layered on top
//! later if needed.
//!
//! Endpoint: `https://{lang}.wikipedia.org/api/rest_v1/page/summary/{title}`

use tokimo_core::{CoreError, CoreResult};

/// Build the canonical cache key: `"{lang}:{lower(title)}"`.
pub fn cache_key(lang: &str, title: &str) -> String {
    format!("{}:{}", lang, title.to_lowercase())
}

/// Fetch the REST summary JSON for a given language + article title.
///
/// Returns the raw JSON body. 404 is mapped to `CoreError::NotFound`;
/// other non-2xx statuses to `CoreError::Provider`.
pub async fn fetch_summary(http: &reqwest::Client, lang: &str, title: &str) -> CoreResult<serde_json::Value> {
    let base = reqwest::Url::parse(&format!("https://{lang}.wikipedia.org/api/rest_v1/page/summary/"))
        .map_err(|e| CoreError::Provider(format!("wikipedia url parse: {e}")))?;
    let url = base
        .join(&title.replace(' ', "_"))
        .map_err(|e| CoreError::Provider(format!("wikipedia url join: {e}")))?;

    let resp = http.get(url).send().await.map_err(CoreError::Upstream)?;
    let status = resp.status();
    if status.as_u16() == 404 {
        return Err(CoreError::NotFound);
    }
    if !status.is_success() {
        return Err(CoreError::Provider(format!(
            "wikipedia returned status {} for {}/{}",
            status, lang, title
        )));
    }
    resp.json::<serde_json::Value>().await.map_err(CoreError::Upstream)
}

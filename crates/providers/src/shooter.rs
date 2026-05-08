//! Shooter (射手网) subtitle search adapter.
//!
//! Endpoint: `POST https://www.shooter.cn/api/subapi.php`
//! Form fields: `filehash`, `pathinfo`, `format=json`, `lang` (e.g. `chn` / `eng`).
//! No API key. Returns a JSON array of subtitles, or the literal `-1` /
//! empty body when nothing matches — both mapped to an empty array here.

use tokimo_core::{CoreError, CoreResult};

const SHOOTER_API_URL: &str = "https://www.shooter.cn/api/subapi.php";

/// Build the cache key from query parameters.
pub fn cache_key(file_hash: &str, path_info: &str, lang: &str) -> String {
    format!(
        "shooter:{}:{}:{}",
        lang.trim().to_lowercase(),
        file_hash.trim().to_lowercase(),
        path_info.trim().to_lowercase()
    )
}

/// Search Shooter for subtitles matching the given file hash.
pub async fn search(
    http: &reqwest::Client,
    file_hash: &str,
    path_info: &str,
    lang: &str,
) -> CoreResult<serde_json::Value> {
    let params = [
        ("filehash", file_hash),
        ("pathinfo", path_info),
        ("format", "json"),
        ("lang", lang),
    ];

    let resp = http
        .post(SHOOTER_API_URL)
        .form(&params)
        .send()
        .await
        .map_err(CoreError::Upstream)?;

    let status = resp.status();
    if !status.is_success() {
        return Err(CoreError::Provider(format!("shooter returned status {status}")));
    }

    let body = resp.text().await.map_err(CoreError::Upstream)?;
    let trimmed = body.trim();

    // Shooter returns the literal `-1` or an empty body when there are no matches.
    if trimmed.is_empty() || trimmed == "-1" {
        return Ok(serde_json::json!([]));
    }

    serde_json::from_str(trimmed).map_err(|e| CoreError::Provider(format!("shooter parse error: {e}")))
}

//! Hitokoto (一言) adapter.
//!
//! Endpoint: `https://v1.hitokoto.cn/?c={c}` — returns a random sentence of a
//! given category (`a` 动画, `b` 漫画, `c` 游戏, `d` 文学, `e` 原创, `f` 来自网络,
//! `g` 其他, `h` 影视, `i` 诗词, `j` 网易云, `k` 哲学, `l` 抖机灵).
//! When `c` is omitted upstream returns a random pick across all categories.

use tokimo_core::{CoreError, CoreResult};

pub const HITOKOTO_BASE_URL: &str = "https://v1.hitokoto.cn/";

/// Build the cache key. `None` (random) => `"random"`.
pub fn cache_key(c: Option<&str>) -> String {
    format!("hitokoto:{}", c.unwrap_or("random"))
}

/// Validate `c` parameter — single character a-l.
pub fn is_valid_category(c: &str) -> bool {
    c.len() == 1 && matches!(c.chars().next(), Some('a'..='l'))
}

/// Fetch a sentence (raw JSON) from Hitokoto.
pub async fn fetch_sentence(http: &reqwest::Client, c: Option<&str>) -> CoreResult<serde_json::Value> {
    let url = match c {
        Some(cat) => format!("{HITOKOTO_BASE_URL}?c={cat}"),
        None => HITOKOTO_BASE_URL.to_string(),
    };

    let resp = http.get(&url).send().await.map_err(CoreError::Upstream)?;
    let status = resp.status();
    if !status.is_success() {
        return Err(CoreError::Provider(format!("hitokoto returned status {status}")));
    }
    resp.json::<serde_json::Value>().await.map_err(CoreError::Upstream)
}

use serde::{Deserialize, Serialize};

/// Compute storage key with namespace and SHA256 hash
pub fn compute_storage_key(ns: &str, sha256_hex: &str, ext: &str) -> String {
    let prefix = &sha256_hex[0..2];
    format!("{}/{}/{}.{}", ns, prefix, sha256_hex, ext)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotItem {
    pub title: String,
    pub url: String,
    pub hot_value: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SportMatch {
    pub match_name: String,
    pub match_date: String,
    pub start_time: Option<String>,
    pub status: Option<String>,
    pub vs_line: Option<String>,
    pub left_team: Option<serde_json::Value>,
    pub right_team: Option<serde_json::Value>,
    pub game: Option<String>,
    pub link: Option<String>,
    pub has_live: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_storage_key() {
        let sha = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let key = compute_storage_key("tmdb", sha, "jpg");
        assert_eq!(
            key,
            "tmdb/ab/abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890.jpg"
        );
    }
}

//! Common helpers shared across upstream provider adapters.
//!
//! Centralizes:
//! - HTTP GET → JSON with consistent error mapping
//! - Generic blob download → Storage with content-type sniffing + sha256
//!   keyed storage (mirrors the original tmdb::download_image logic).

use bytes::Bytes;
use sha2::{Digest, Sha256};
use tokimo_core::{compute_storage_key, CoreError, CoreResult, Storage};

/// HTTP GET that returns parsed JSON, mapping errors to `CoreError`.
///
/// Non-2xx upstream responses are returned as `CoreError::Provider` with the
/// status code; transport-level reqwest errors map to `CoreError::Upstream`.
pub async fn http_get_json(client: &reqwest::Client, url: &str) -> CoreResult<serde_json::Value> {
    let response = client.get(url).send().await.map_err(CoreError::Upstream)?;
    if !response.status().is_success() {
        return Err(CoreError::Provider(format!(
            "Upstream {} returned status {}",
            url,
            response.status()
        )));
    }
    response.json::<serde_json::Value>().await.map_err(CoreError::Upstream)
}

/// Download an absolute URL into the configured storage backend.
///
/// Returns `(sha256_hex, storage_key)`. The key is namespaced by `ns` and
/// derived from the SHA-256 of the bytes, so identical content from different
/// providers is deduplicated within their namespace.
pub async fn download_to_storage(
    client: &reqwest::Client,
    url: &str,
    storage: &dyn Storage,
    ns: &str,
) -> CoreResult<(String, String)> {
    let response = client.get(url).send().await.map_err(CoreError::Upstream)?;
    if !response.status().is_success() {
        return Err(CoreError::Provider(format!(
            "Failed to download {} ({})",
            url,
            response.status()
        )));
    }
    let bytes = response.bytes().await.map_err(CoreError::Upstream)?;

    let mime = infer::get(&bytes)
        .map(|t| t.mime_type().to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let ext = mime.split('/').nth(1).unwrap_or("bin");

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let sha256_hex = hex::encode(hasher.finalize());

    let storage_key = compute_storage_key(ns, &sha256_hex, ext);
    storage.put(&storage_key, Bytes::from(bytes.to_vec()), &mime).await?;

    Ok((sha256_hex, storage_key))
}

use axum::{
    body::Body,
    extract::Request,
    http::{header, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::Response,
};
use xxhash_rust::xxh3::xxh3_64;

const DEFAULT_CACHE_TTL: u64 = 300;

/// Middleware that adds cache control headers and ETag to successful GET/HEAD responses.
///
/// - Applies only to 2xx responses without existing Cache-Control
/// - Adds Cache-Control: public, max-age=<N>, stale-while-revalidate=<N/2>
/// - TTL comes from X-Cache-TTL header (if present) or DEFAULT_CACHE_TTL
/// - Computes ETag from response body using xxhash
/// - Handles If-None-Match to return 304 Not Modified when appropriate
/// - Removes X-Cache-TTL from final response (internal header)
/// - Skips internal/admin routes: /admin/*, /health, /_internal/*
///
/// Note: Provider handlers can gradually migrate to set X-Cache-TTL for custom TTLs.
pub async fn cache_headers(req: Request, next: Next) -> Result<Response, StatusCode> {
    // Skip internal and admin routes
    if should_skip_path(req.uri().path()) {
        return Ok(next.run(req).await);
    }

    let method = req.method().clone();
    let if_none_match = req
        .headers()
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Only apply to GET and HEAD methods
    if method != Method::GET && method != Method::HEAD {
        return Ok(next.run(req).await);
    }

    let response = next.run(req).await;

    // Only process successful 2xx responses
    let status = response.status();
    if !status.is_success() {
        return Ok(response);
    }

    // Skip if Cache-Control already exists
    if response.headers().contains_key(header::CACHE_CONTROL) {
        return Ok(response);
    }

    // Extract X-Cache-TTL if present
    let ttl = response
        .headers()
        .get("x-cache-ttl")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_CACHE_TTL);

    // Split response into parts
    let (mut parts, body) = response.into_parts();

    // Remove internal X-Cache-TTL header
    parts.headers.remove("x-cache-ttl");

    // Convert body to bytes for ETag computation
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            // Body aggregation failed (e.g., stream error, size limit)
            // Return original status without cache headers, preserve body error state
            tracing::debug!("Body aggregation failed, skipping cache headers: {}", e);
            return Ok(Response::from_parts(parts, Body::empty()));
        }
    };

    // Compute ETag from body
    let hash = xxh3_64(&body_bytes);
    let etag_value = format!("\"{hash:x}\"");

    // Check If-None-Match for 304 response
    if let Some(if_none_match_value) = if_none_match {
        if etag_matches(&if_none_match_value, &etag_value) {
            // Return 304 Not Modified with empty body
            parts.status = StatusCode::NOT_MODIFIED;
            parts.headers.remove(header::CONTENT_LENGTH);
            parts.headers.remove(header::CONTENT_TYPE);

            // Add ETag and Cache-Control headers
            if let Ok(etag) = HeaderValue::from_str(&etag_value) {
                parts.headers.insert(header::ETAG, etag);
            }

            let cache_control = format!("public, max-age={}, stale-while-revalidate={}", ttl, ttl / 2);
            if let Ok(cc) = HeaderValue::from_str(&cache_control) {
                parts.headers.insert(header::CACHE_CONTROL, cc);
            }

            return Ok(Response::from_parts(parts, Body::empty()));
        }
    }

    // Add Cache-Control header
    let cache_control = format!("public, max-age={}, stale-while-revalidate={}", ttl, ttl / 2);
    if let Ok(cc) = HeaderValue::from_str(&cache_control) {
        parts.headers.insert(header::CACHE_CONTROL, cc);
    }

    // Add ETag header
    if let Ok(etag) = HeaderValue::from_str(&etag_value) {
        parts.headers.insert(header::ETAG, etag);
    }

    // Handle HEAD method - return empty body with headers
    if method == Method::HEAD {
        // Update Content-Length to match what would have been sent
        if let Ok(len) = HeaderValue::from_str(&body_bytes.len().to_string()) {
            parts.headers.insert(header::CONTENT_LENGTH, len);
        }
        return Ok(Response::from_parts(parts, Body::empty()));
    }

    // Return response with body for GET
    Ok(Response::from_parts(parts, Body::from(body_bytes)))
}

/// Check if the request path should skip cache header processing.
///
/// Skips internal and admin routes:
/// - /health or /api/health
/// - /admin, /api/admin, /admin/*, /api/admin/*
/// - /_internal, /api/_internal, /_internal/*, /api/_internal/*
///
/// Tolerates both full paths (/api/...) and relative paths (/...) since
/// api_routes may be nested under /api prefix in the main app.
fn should_skip_path(path: &str) -> bool {
    // Normalize: strip leading /api if present
    let normalized = path.strip_prefix("/api").unwrap_or(path);

    // Check skip patterns
    normalized == "/health"
        || normalized == "/admin"
        || normalized.starts_with("/admin/")
        || normalized == "/_internal"
        || normalized.starts_with("/_internal/")
}

/// Check if the If-None-Match value matches the ETag.
///
/// Supports:
/// - Exact match: "abc123"
/// - Multiple tags: "abc123", "def456"
/// - Wildcard: *
/// - Weak tags: W/"abc123" (treated as exact match for simplicity)
fn etag_matches(if_none_match: &str, etag: &str) -> bool {
    // Wildcard match
    if if_none_match.trim() == "*" {
        return true;
    }

    // Split by comma and check each tag
    for tag in if_none_match.split(',') {
        let tag = tag.trim();

        // Handle weak tags by stripping W/ prefix
        let normalized_tag = tag.strip_prefix("W/").unwrap_or(tag);

        // Exact match (with or without quotes)
        if normalized_tag == etag || normalized_tag.trim_matches('"') == etag.trim_matches('"') {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etag_matches_exact() {
        assert!(etag_matches("\"abc123\"", "\"abc123\""));
        assert!(!etag_matches("\"abc123\"", "\"def456\""));
    }

    #[test]
    fn test_etag_matches_wildcard() {
        assert!(etag_matches("*", "\"abc123\""));
    }

    #[test]
    fn test_etag_matches_multiple() {
        assert!(etag_matches("\"abc\", \"def\", \"ghi\"", "\"def\""));
        assert!(!etag_matches("\"abc\", \"def\", \"ghi\"", "\"xyz\""));
    }

    #[test]
    fn test_etag_matches_weak() {
        assert!(etag_matches("W/\"abc123\"", "\"abc123\""));
    }

    #[test]
    fn test_should_skip_path_health() {
        assert!(should_skip_path("/health"));
        assert!(should_skip_path("/api/health"));
    }

    #[test]
    fn test_should_skip_path_admin() {
        assert!(should_skip_path("/admin"));
        assert!(should_skip_path("/api/admin"));
        assert!(should_skip_path("/admin/users"));
        assert!(should_skip_path("/api/admin/users"));
        assert!(should_skip_path("/admin/config/keys"));
        assert!(should_skip_path("/api/admin/config/keys"));
    }

    #[test]
    fn test_should_skip_path_internal() {
        assert!(should_skip_path("/_internal"));
        assert!(should_skip_path("/api/_internal"));
        assert!(should_skip_path("/_internal/metrics"));
        assert!(should_skip_path("/api/_internal/metrics"));
    }

    #[test]
    fn test_should_not_skip_provider_routes() {
        assert!(!should_skip_path("/tmdb/search"));
        assert!(!should_skip_path("/api/tmdb/search"));
        assert!(!should_skip_path("/omdb/movie/tt1234567"));
        assert!(!should_skip_path("/api/spotify/track/123"));
    }
}

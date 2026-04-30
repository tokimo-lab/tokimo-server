use crate::{Bytes, CoreResult};
use async_trait::async_trait;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

/// Storage backend trait for persisting binary assets
#[async_trait]
pub trait Storage: Send + Sync {
    /// Store bytes at the given key
    async fn put(&self, key: &str, data: Bytes, content_type: &str) -> CoreResult<()>;

    /// Delete object at key
    async fn delete(&self, key: &str) -> CoreResult<()>;

    /// Generate a URL for the given key.
    ///
    /// For public buckets this returns the assembled public URL.
    /// For private buckets this returns a presigned GET URL with a backend-defined TTL.
    /// The operation is async because presigning may require signing logic.
    async fn url_for(&self, key: &str) -> CoreResult<String>;

    /// Check if key exists
    async fn exists(&self, key: &str) -> CoreResult<bool>;
}

/// Cache trait with namespace support
#[async_trait]
pub trait Cache: Send + Sync {
    /// Get cached value
    async fn get(&self, namespace: &str, key: &str) -> CoreResult<Option<Bytes>>;

    /// Set cached value with TTL
    async fn set(&self, namespace: &str, key: &str, value: Bytes, ttl: Duration) -> CoreResult<()>;

    /// Delete cached value
    async fn delete(&self, namespace: &str, key: &str) -> CoreResult<()>;
}

/// Single-flight mechanism to prevent duplicate concurrent requests
#[async_trait]
pub trait SingleFlight: Send + Sync {
    /// Execute function once for a given key, deduplicate concurrent calls
    async fn do_once<T, F, Fut>(&self, key: &str, f: F) -> CoreResult<T>
    where
        T: Clone + Send + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = CoreResult<T>> + Send + 'static;
}

/// Rate limiter with token bucket algorithm
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Acquire a token, blocking if necessary
    async fn acquire(&self, provider: &str) -> CoreResult<()>;

    /// Try to acquire without blocking
    async fn try_acquire(&self, provider: &str) -> CoreResult<bool>;
}

/// Marker trait for providers
pub trait Provider: Send + Sync {}

impl<T> Provider for Arc<T> where T: Provider {}

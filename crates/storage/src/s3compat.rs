//! Unified S3-compatible storage backend.
//!
//! Powers both the `s3` (AWS S3 / MinIO / generic S3-compatible) and `oss`
//! (Aliyun OSS, which speaks S3-compatible protocol) features. The only
//! per-backend difference is the endpoint URL — Aliyun OSS uses
//! `https://oss-cn-<region>.aliyuncs.com`, AWS S3 uses the default endpoint
//! derived from the region, and MinIO uses an arbitrary HTTP endpoint.
//!
//! ## URL generation policy
//!
//! Controlled by `presign_ttl_seconds`:
//! - `0` → public bucket, returns `{public_base}/{key}` synchronously.
//! - `> 0` → private bucket, returns a presigned GET URL valid for that
//!   many seconds.

use async_trait::async_trait;
use object_store::{
    aws::{AmazonS3, AmazonS3Builder},
    path::Path as ObjectPath,
    signer::Signer,
    ObjectStore, PutPayload,
};
use std::time::Duration;
use tokimo_core::{Bytes, CoreError, CoreResult, Storage};

/// Configuration for an S3-compatible storage backend.
#[derive(Debug, Clone)]
pub struct S3CompatConfig {
    /// Optional custom endpoint URL.
    /// - `None` → use AWS S3 (default endpoint is derived from region).
    /// - `Some("http://localhost:9000")` → MinIO.
    /// - `Some("https://oss-cn-hangzhou.aliyuncs.com")` → Aliyun OSS.
    pub endpoint: Option<String>,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    /// Public URL prefix used when `presign_ttl_seconds == 0`. Required in
    /// that case; ignored otherwise.
    pub public_base: Option<String>,
    /// `0` → public bucket, return `{public_base}/{key}`.
    /// `> 0` → private bucket, return a presigned GET URL with this TTL.
    pub presign_ttl_seconds: u64,
    /// If true, use path-style addressing (`https://endpoint/{bucket}/{key}`).
    /// Required for MinIO and many S3-compatible services. Defaults to true
    /// when an explicit endpoint is set, false for plain AWS S3.
    pub virtual_hosted_style: bool,
}

/// S3-compatible storage backend powered by `object_store::aws::AmazonS3`.
pub struct S3CompatStorage {
    inner: AmazonS3,
    public_base: Option<String>,
    presign_ttl_seconds: u64,
}

impl S3CompatStorage {
    pub fn new(config: S3CompatConfig) -> CoreResult<Self> {
        if config.presign_ttl_seconds == 0 && config.public_base.is_none() {
            return Err(CoreError::Storage(
                "public_base is required when presign_ttl_seconds == 0".into(),
            ));
        }

        let mut builder = AmazonS3Builder::new()
            .with_region(&config.region)
            .with_bucket_name(&config.bucket)
            .with_access_key_id(&config.access_key_id)
            .with_secret_access_key(&config.secret_access_key);

        if let Some(endpoint) = &config.endpoint {
            builder = builder.with_endpoint(endpoint);
            // Plain HTTP endpoints (MinIO/dev) need allow_http enabled.
            if endpoint.starts_with("http://") {
                builder = builder.with_allow_http(true);
            }
        }

        builder = builder.with_virtual_hosted_style_request(config.virtual_hosted_style);

        let inner = builder
            .build()
            .map_err(|e| CoreError::Storage(format!("failed to build S3 client: {e}")))?;

        Ok(Self {
            inner,
            public_base: config.public_base,
            presign_ttl_seconds: config.presign_ttl_seconds,
        })
    }
}

fn parse_path(key: &str) -> CoreResult<ObjectPath> {
    ObjectPath::parse(key).map_err(|e| CoreError::Storage(format!("invalid storage key {key}: {e}")))
}

#[async_trait]
impl Storage for S3CompatStorage {
    async fn put(&self, key: &str, data: Bytes, _content_type: &str) -> CoreResult<()> {
        let path = parse_path(key)?;
        let payload = PutPayload::from_bytes(data);
        self.inner
            .put(&path, payload)
            .await
            .map_err(|e| CoreError::Storage(format!("S3 put {key} failed: {e}")))?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> CoreResult<()> {
        let path = parse_path(key)?;
        match self.inner.delete(&path).await {
            Ok(_) => Ok(()),
            Err(object_store::Error::NotFound { .. }) => Ok(()),
            Err(e) => Err(CoreError::Storage(format!("S3 delete {key} failed: {e}"))),
        }
    }

    async fn url_for(&self, key: &str) -> CoreResult<String> {
        if self.presign_ttl_seconds == 0 {
            let base = self
                .public_base
                .as_deref()
                .ok_or_else(|| CoreError::Storage("public_base is required for non-presigned URLs".into()))?;
            return Ok(format!("{}/{}", base.trim_end_matches('/'), key));
        }

        let path = parse_path(key)?;
        let url = self
            .inner
            .signed_url(
                reqwest::Method::GET,
                &path,
                Duration::from_secs(self.presign_ttl_seconds),
            )
            .await
            .map_err(|e| CoreError::Storage(format!("S3 presign {key} failed: {e}")))?;
        Ok(url.into())
    }

    async fn exists(&self, key: &str) -> CoreResult<bool> {
        let path = parse_path(key)?;
        match self.inner.head(&path).await {
            Ok(_) => Ok(true),
            Err(object_store::Error::NotFound { .. }) => Ok(false),
            Err(e) => Err(CoreError::Storage(format!("S3 head {key} failed: {e}"))),
        }
    }
}

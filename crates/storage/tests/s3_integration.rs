//! Integration tests for the S3-compatible storage backend.
//!
//! These tests are gated behind the `s3` feature **and** a runtime check for
//! the `MINIO_ENDPOINT` environment variable so local `cargo test` runs
//! without MinIO simply skip them. CI brings up MinIO and sets the env.
//!
//! Required env vars (set by CI):
//! - `MINIO_ENDPOINT` (e.g. `http://localhost:9000`)
//! - `MINIO_ACCESS_KEY` (default `minioadmin`)
//! - `MINIO_SECRET_KEY` (default `minioadmin`)
//! - `MINIO_BUCKET` (default `tokimo-test`)

#![cfg(feature = "s3")]

use tokimo_core::{Bytes, Storage};
use tokimo_storage::{S3CompatConfig, S3CompatStorage};

fn env_or_skip() -> Option<(String, String, String, String)> {
    let endpoint = std::env::var("MINIO_ENDPOINT").ok()?;
    let access = std::env::var("MINIO_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".into());
    let secret = std::env::var("MINIO_SECRET_KEY").unwrap_or_else(|_| "minioadmin".into());
    let bucket = std::env::var("MINIO_BUCKET").unwrap_or_else(|_| "tokimo-test".into());
    Some((endpoint, access, secret, bucket))
}

fn build_storage(presign_ttl_seconds: u64) -> Option<(S3CompatStorage, String, String)> {
    let (endpoint, access_key_id, secret_access_key, bucket) = env_or_skip()?;
    let public_base = format!("{}/{}", endpoint.trim_end_matches('/'), bucket);
    let config = S3CompatConfig {
        endpoint: Some(endpoint.clone()),
        region: "us-east-1".into(),
        bucket: bucket.clone(),
        access_key_id,
        secret_access_key,
        public_base: Some(public_base.clone()),
        presign_ttl_seconds,
        virtual_hosted_style: false,
    };
    let storage = S3CompatStorage::new(config).expect("build S3CompatStorage");
    Some((storage, public_base, bucket))
}

#[tokio::test]
async fn test_minio_round_trip_public() {
    let Some((storage, _public_base, _bucket)) = build_storage(0) else {
        eprintln!("MINIO_ENDPOINT not set, skipping");
        return;
    };

    let key = format!(
        "test/public-{}.txt",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let payload = Bytes::from_static(b"hello tokimo public");

    storage.put(&key, payload.clone(), "text/plain").await.expect("put");
    assert!(storage.exists(&key).await.expect("exists"));

    let url = storage.url_for(&key).await.expect("url_for");
    assert!(url.contains(&key), "URL should contain key, got {url}");

    let body = reqwest::get(&url)
        .await
        .expect("http get")
        .error_for_status()
        .expect("2xx")
        .bytes()
        .await
        .expect("body");
    assert_eq!(body.as_ref(), payload.as_ref());

    storage.delete(&key).await.expect("delete");
    assert!(!storage.exists(&key).await.expect("exists after delete"));
}

#[tokio::test]
async fn test_minio_presigned_get() {
    let Some((storage, _, _)) = build_storage(300) else {
        eprintln!("MINIO_ENDPOINT not set, skipping");
        return;
    };

    let key = format!(
        "test/presigned-{}.txt",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let payload = Bytes::from_static(b"hello tokimo presigned");

    storage.put(&key, payload.clone(), "text/plain").await.expect("put");

    let url = storage.url_for(&key).await.expect("presigned url");
    assert!(
        url.contains("X-Amz-Signature"),
        "presigned URL must contain X-Amz-Signature, got {url}"
    );

    let body = reqwest::get(&url)
        .await
        .expect("http get presigned")
        .error_for_status()
        .expect("2xx")
        .bytes()
        .await
        .expect("body");
    assert_eq!(body.as_ref(), payload.as_ref());

    storage.delete(&key).await.expect("delete");
}

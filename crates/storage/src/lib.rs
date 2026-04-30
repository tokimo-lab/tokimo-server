mod local;

#[cfg(any(feature = "s3", feature = "oss"))]
mod s3compat;

pub use local::LocalStorage;

#[cfg(any(feature = "s3", feature = "oss"))]
pub use s3compat::{S3CompatConfig, S3CompatStorage};

use std::sync::Arc;
use tokimo_core::{CoreResult, Storage};

/// Factory to create storage backend from environment variables
pub async fn storage_from_env() -> CoreResult<Arc<dyn Storage>> {
    let backend = std::env::var("STORAGE_BACKEND").unwrap_or_else(|_| "local".to_string());

    match backend.as_str() {
        "local" => {
            let root = std::env::var("STORAGE_LOCAL_ROOT")
                .map_err(|_| tokimo_core::CoreError::Storage("STORAGE_LOCAL_ROOT not set".into()))?;
            let public_base = std::env::var("STORAGE_LOCAL_PUBLIC_BASE")
                .map_err(|_| tokimo_core::CoreError::Storage("STORAGE_LOCAL_PUBLIC_BASE not set".into()))?;

            Ok(Arc::new(LocalStorage::new(root.into(), public_base)))
        }
        #[cfg(feature = "s3")]
        "s3" => Ok(Arc::new(s3compat_from_env("S3", None)?)),
        #[cfg(feature = "oss")]
        "oss" => Ok(Arc::new(s3compat_from_env(
            "OSS",
            Some("https://oss-cn-hangzhou.aliyuncs.com".into()),
        )?)),
        _ => Err(tokimo_core::CoreError::Storage(format!(
            "Unknown storage backend: {}",
            backend
        ))),
    }
}

#[cfg(any(feature = "s3", feature = "oss"))]
fn s3compat_from_env(prefix: &str, default_endpoint: Option<String>) -> CoreResult<S3CompatStorage> {
    let env = |k: &str| std::env::var(format!("STORAGE_{prefix}_{k}"));
    let req = |k: &str| env(k).map_err(|_| tokimo_core::CoreError::Storage(format!("STORAGE_{prefix}_{k} not set")));

    let bucket = req("BUCKET")?;
    let region = env("REGION").unwrap_or_else(|_| "us-east-1".into());
    let endpoint = env("ENDPOINT").ok().or(default_endpoint);
    let access_key_id = req("ACCESS_KEY_ID")?;
    let secret_access_key = env("SECRET_ACCESS_KEY")
        .or_else(|_| env("ACCESS_KEY_SECRET"))
        .map_err(|_| {
            tokimo_core::CoreError::Storage(format!(
                "STORAGE_{prefix}_SECRET_ACCESS_KEY (or ACCESS_KEY_SECRET) not set"
            ))
        })?;
    let public_base = env("PUBLIC_BASE").ok();
    let presign_ttl_seconds = env("PRESIGN_TTL_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    if presign_ttl_seconds == 0 && public_base.is_none() {
        return Err(tokimo_core::CoreError::Storage(format!(
            "STORAGE_{prefix}_PUBLIC_BASE is required when STORAGE_{prefix}_PRESIGN_TTL_SECONDS=0"
        )));
    }

    // Path-style addressing for any custom endpoint (MinIO, OSS dev, etc.).
    let virtual_hosted_style = endpoint.is_none();

    S3CompatStorage::new(S3CompatConfig {
        endpoint,
        region,
        bucket,
        access_key_id,
        secret_access_key,
        public_base,
        presign_ttl_seconds,
        virtual_hosted_style,
    })
}

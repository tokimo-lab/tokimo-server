mod local;

#[cfg(feature = "s3")]
mod s3;

#[cfg(feature = "oss")]
mod oss;

pub use local::LocalStorage;

#[cfg(feature = "s3")]
pub use s3::S3Storage;

#[cfg(feature = "oss")]
pub use oss::OssStorage;

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
        "s3" => {
            let bucket = std::env::var("STORAGE_S3_BUCKET")
                .map_err(|_| tokimo_core::CoreError::Storage("STORAGE_S3_BUCKET not set".into()))?;
            let region = std::env::var("STORAGE_S3_REGION")
                .map_err(|_| tokimo_core::CoreError::Storage("STORAGE_S3_REGION not set".into()))?;
            let public_base = std::env::var("STORAGE_S3_PUBLIC_BASE")
                .map_err(|_| tokimo_core::CoreError::Storage("STORAGE_S3_PUBLIC_BASE not set".into()))?;

            Ok(Arc::new(S3Storage::new(bucket, region, public_base)))
        }
        #[cfg(feature = "oss")]
        "oss" => {
            let bucket = std::env::var("STORAGE_OSS_BUCKET")
                .map_err(|_| tokimo_core::CoreError::Storage("STORAGE_OSS_BUCKET not set".into()))?;
            let region = std::env::var("STORAGE_OSS_REGION")
                .map_err(|_| tokimo_core::CoreError::Storage("STORAGE_OSS_REGION not set".into()))?;
            let public_base = std::env::var("STORAGE_OSS_PUBLIC_BASE")
                .map_err(|_| tokimo_core::CoreError::Storage("STORAGE_OSS_PUBLIC_BASE not set".into()))?;

            Ok(Arc::new(OssStorage::new(bucket, region, public_base)))
        }
        _ => Err(tokimo_core::CoreError::Storage(format!(
            "Unknown storage backend: {}",
            backend
        ))),
    }
}

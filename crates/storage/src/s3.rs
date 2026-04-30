use async_trait::async_trait;
use tokimo_core::{Bytes, CoreResult, Storage};

pub struct S3Storage {
    _bucket: String,
    _region: String,
    public_base: String,
}

impl S3Storage {
    pub fn new(bucket: String, region: String, public_base: String) -> Self {
        Self {
            _bucket: bucket,
            _region: region,
            public_base,
        }
    }
}

#[async_trait]
impl Storage for S3Storage {
    async fn put(&self, _key: &str, _data: Bytes, _content_type: &str) -> CoreResult<()> {
        // TODO: Implement S3 put using aws-sdk-s3 or rusoto
        unimplemented!("S3 storage not yet implemented")
    }

    async fn delete(&self, _key: &str) -> CoreResult<()> {
        // TODO: Implement S3 delete
        unimplemented!("S3 storage not yet implemented")
    }

    async fn url_for(&self, key: &str) -> CoreResult<String> {
        Ok(format!("{}/{}", self.public_base.trim_end_matches('/'), key))
    }

    async fn exists(&self, _key: &str) -> CoreResult<bool> {
        // TODO: Implement S3 head_object
        unimplemented!("S3 storage not yet implemented")
    }
}

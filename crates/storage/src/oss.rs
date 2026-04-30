use async_trait::async_trait;
use tokimo_core::{Bytes, CoreResult, Storage};

pub struct OssStorage {
    _bucket: String,
    _region: String,
    public_base: String,
}

impl OssStorage {
    pub fn new(bucket: String, region: String, public_base: String) -> Self {
        Self {
            _bucket: bucket,
            _region: region,
            public_base,
        }
    }
}

#[async_trait]
impl Storage for OssStorage {
    async fn put(&self, _key: &str, _data: Bytes, _content_type: &str) -> CoreResult<()> {
        // TODO: Implement Aliyun OSS put using aliyun-oss-rs or similar
        unimplemented!("OSS storage not yet implemented")
    }

    async fn delete(&self, _key: &str) -> CoreResult<()> {
        // TODO: Implement OSS delete
        unimplemented!("OSS storage not yet implemented")
    }

    async fn url_for(&self, key: &str) -> CoreResult<String> {
        Ok(format!("{}/{}", self.public_base.trim_end_matches('/'), key))
    }

    async fn exists(&self, _key: &str) -> CoreResult<bool> {
        // TODO: Implement OSS head_object
        unimplemented!("OSS storage not yet implemented")
    }
}

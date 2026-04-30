use async_trait::async_trait;
use std::path::PathBuf;
use tokimo_core::{Bytes, CoreResult, Storage};
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub struct LocalStorage {
    root: PathBuf,
    public_base: String,
}

impl LocalStorage {
    pub fn new(root: PathBuf, public_base: String) -> Self {
        Self { root, public_base }
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn put(&self, key: &str, data: Bytes, _content_type: &str) -> CoreResult<()> {
        let path = self.root.join(key);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut file = fs::File::create(&path).await?;
        file.write_all(&data).await?;
        file.flush().await?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> CoreResult<()> {
        let path = self.root.join(key);
        if path.exists() {
            fs::remove_file(path).await?;
        }
        Ok(())
    }

    fn url_for(&self, key: &str) -> String {
        format!("{}/{}", self.public_base.trim_end_matches('/'), key)
    }

    async fn exists(&self, key: &str) -> CoreResult<bool> {
        let path = self.root.join(key);
        Ok(path.exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_local_storage_round_trip() {
        let tmp = TempDir::new().unwrap();
        let storage = LocalStorage::new(tmp.path().to_path_buf(), "http://localhost/assets".into());

        let data = Bytes::from_static(b"hello world");
        storage.put("test/file.txt", data.clone(), "text/plain").await.unwrap();

        assert!(storage.exists("test/file.txt").await.unwrap());

        let url = storage.url_for("test/file.txt");
        assert_eq!(url, "http://localhost/assets/test/file.txt");

        storage.delete("test/file.txt").await.unwrap();
        assert!(!storage.exists("test/file.txt").await.unwrap());
    }
}

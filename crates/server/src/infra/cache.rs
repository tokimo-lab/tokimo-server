use async_trait::async_trait;
use bytes::Bytes;
use sea_orm::{entity::*, query::*, DatabaseConnection, EntityTrait};
use std::time::Duration;
use tokimo_core::{Cache, CoreError, CoreResult};

use crate::db::entities::{cache_entries, CacheEntries};

pub struct PgCache {
    db: DatabaseConnection,
}

impl PgCache {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Cache for PgCache {
    async fn get(&self, namespace: &str, key: &str) -> CoreResult<Option<Bytes>> {
        let now = chrono::Utc::now();

        let result = CacheEntries::find()
            .filter(cache_entries::Column::Namespace.eq(namespace))
            .filter(cache_entries::Column::Key.eq(key))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::Cache(e.to_string()))?;

        if let Some(entry) = result {
            if entry.expires_at.naive_utc() > now.naive_utc() {
                return Ok(Some(Bytes::from(entry.value)));
            }

            // Expired, delete it
            CacheEntries::delete_many()
                .filter(cache_entries::Column::Namespace.eq(namespace))
                .filter(cache_entries::Column::Key.eq(key))
                .exec(&self.db)
                .await
                .map_err(|e| CoreError::Cache(e.to_string()))?;
        }

        Ok(None)
    }

    async fn set(&self, namespace: &str, key: &str, value: Bytes, ttl: Duration) -> CoreResult<()> {
        let expires_at = chrono::Utc::now()
            + chrono::Duration::from_std(ttl).map_err(|e| CoreError::Cache(format!("Invalid TTL: {}", e)))?;

        let model = cache_entries::ActiveModel {
            namespace: Set(namespace.to_string()),
            key: Set(key.to_string()),
            value: Set(value.to_vec()),
            expires_at: Set(expires_at.into()),
        };

        CacheEntries::insert(model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([cache_entries::Column::Namespace, cache_entries::Column::Key])
                    .update_columns([cache_entries::Column::Value, cache_entries::Column::ExpiresAt])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::Cache(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, namespace: &str, key: &str) -> CoreResult<()> {
        CacheEntries::delete_many()
            .filter(cache_entries::Column::Namespace.eq(namespace))
            .filter(cache_entries::Column::Key.eq(key))
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::Cache(e.to_string()))?;

        Ok(())
    }
}

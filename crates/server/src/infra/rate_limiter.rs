use async_trait::async_trait;
use sea_orm::{entity::*, query::*, DatabaseConnection, EntityTrait, TransactionTrait};
use std::time::Duration;
use tokimo_core::{CoreError, CoreResult, RateLimiter};
use tokio::time::sleep;

use crate::db::entities::{rate_limit_buckets, RateLimitBuckets};

pub struct PgRateLimiter {
    db: DatabaseConnection,
}

impl PgRateLimiter {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn acquire_with_retry(&self, provider: &str, max_retries: usize) -> CoreResult<()> {
        for attempt in 0..max_retries {
            let txn = self.db.begin().await.map_err(|e| CoreError::Database(e.to_string()))?;

            let row = RateLimitBuckets::find()
                .filter(rate_limit_buckets::Column::Provider.eq(provider))
                .lock_exclusive()
                .one(&txn)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?;

            let now = chrono::Utc::now();

            if let Some(bucket) = row {
                let elapsed = (now - bucket.updated_at.naive_utc().and_utc()).num_milliseconds() as f64 / 1000.0;
                let refilled = bucket.refill_rate_per_sec * elapsed;
                let new_tokens = (bucket.tokens + refilled).min(bucket.capacity);

                if new_tokens >= 1.0 {
                    let updated = rate_limit_buckets::ActiveModel {
                        provider: Set(provider.to_string()),
                        tokens: Set(new_tokens - 1.0),
                        capacity: Set(bucket.capacity),
                        refill_rate_per_sec: Set(bucket.refill_rate_per_sec),
                        updated_at: Set(now.into()),
                    };

                    RateLimitBuckets::insert(updated)
                        .on_conflict(
                            sea_orm::sea_query::OnConflict::column(rate_limit_buckets::Column::Provider)
                                .update_columns([
                                    rate_limit_buckets::Column::Tokens,
                                    rate_limit_buckets::Column::UpdatedAt,
                                ])
                                .to_owned(),
                        )
                        .exec(&txn)
                        .await
                        .map_err(|e| CoreError::Database(e.to_string()))?;

                    txn.commit().await.map_err(|e| CoreError::Database(e.to_string()))?;
                    return Ok(());
                } else {
                    txn.commit().await.map_err(|e| CoreError::Database(e.to_string()))?;

                    if attempt < max_retries - 1 {
                        let wait_time = ((1.0 - new_tokens) / bucket.refill_rate_per_sec).max(0.1);
                        sleep(Duration::from_secs_f64(wait_time)).await;
                    }
                }
            } else {
                // No bucket exists, create with default: 10 req/sec, capacity 10
                let new_bucket = rate_limit_buckets::ActiveModel {
                    provider: Set(provider.to_string()),
                    tokens: Set(9.0),
                    capacity: Set(10.0),
                    refill_rate_per_sec: Set(10.0),
                    updated_at: Set(now.into()),
                };

                RateLimitBuckets::insert(new_bucket)
                    .exec(&txn)
                    .await
                    .map_err(|e| CoreError::Database(e.to_string()))?;

                txn.commit().await.map_err(|e| CoreError::Database(e.to_string()))?;
                return Ok(());
            }
        }

        Err(CoreError::RateLimited)
    }
}

#[async_trait]
impl RateLimiter for PgRateLimiter {
    async fn acquire(&self, provider: &str) -> CoreResult<()> {
        self.acquire_with_retry(provider, 3).await
    }

    async fn try_acquire(&self, provider: &str) -> CoreResult<bool> {
        match self.acquire_with_retry(provider, 1).await {
            Ok(()) => Ok(true),
            Err(CoreError::RateLimited) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

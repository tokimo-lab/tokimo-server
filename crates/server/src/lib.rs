pub mod capabilities;
pub mod config;
pub mod db;
pub mod error;
pub mod infra;
pub mod jobs;
pub mod metrics;
pub mod middleware;
pub mod providers_registry;
pub mod routes;

pub use config::AppConfig;
pub use error::{AppError, AppResult};

use crate::infra::PgSingleFlight;
use std::collections::HashMap;
use std::sync::Arc;
use tokimo_core::{Cache, RateLimiter, Storage};
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct ProviderRuntimeConfig {
    pub ttl_seconds: i64,
    pub enabled: bool,
}

#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub storage: Arc<dyn Storage>,
    pub cache: Arc<dyn Cache>,
    pub single_flight: Arc<PgSingleFlight>,
    pub rate_limiter: Arc<dyn RateLimiter>,
    pub http: reqwest::Client,
    pub config: Arc<AppConfig>,
    pub metrics: Arc<crate::metrics::MetricsStore>,
    pub provider_configs: Arc<RwLock<HashMap<String, ProviderRuntimeConfig>>>,
}

impl AppState {
    /// Resolve the effective cache TTL for a provider key.
    ///
    /// Lookup order:
    /// 1. Runtime override loaded from `provider_configs` DB rows
    /// 2. Static `default_ttl_seconds` from [`providers_registry`]
    /// 3. Conservative 12h fallback for unknown keys
    pub async fn provider_ttl(&self, key: &str) -> i64 {
        let g = self.provider_configs.read().await;
        if let Some(c) = g.get(key) {
            return c.ttl_seconds;
        }
        drop(g);
        crate::providers_registry::lookup(key)
            .map(|m| m.default_ttl_seconds)
            .unwrap_or(12 * 60 * 60)
    }

    /// Reload all provider_configs rows into the in-memory cache.
    /// Called at startup and after admin PATCH /api/admin/providers/:key.
    pub async fn reload_provider_configs(&self) -> AppResult<()> {
        use sea_orm::EntityTrait;
        let rows = crate::db::entities::ProviderConfigs::find()
            .all(&self.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let mut g = self.provider_configs.write().await;
        g.clear();
        for r in rows {
            g.insert(
                r.key,
                ProviderRuntimeConfig {
                    ttl_seconds: r.ttl_seconds as i64,
                    enabled: r.enabled,
                },
            );
        }
        Ok(())
    }
}

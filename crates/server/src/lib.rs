pub mod config;
pub mod db;
pub mod error;
pub mod infra;
pub mod metrics;
pub mod middleware;
pub mod routes;

pub use config::AppConfig;
pub use error::{AppError, AppResult};

use crate::infra::PgSingleFlight;
use std::sync::Arc;
use tokimo_core::{Cache, RateLimiter, Storage};

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
}

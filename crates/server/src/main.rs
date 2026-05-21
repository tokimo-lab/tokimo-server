use std::sync::Arc;
use std::time::Duration;

use axum::{http::Method, Router};
use sea_orm::{ConnectOptions, Database};
use tokimo_migration::MigratorTrait;
use tower_http::{
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tokimo_server::metrics::MetricsStore;
use tokimo_server::{infra, routes, AppConfig, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tokimo_server=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Arc::new(AppConfig::from_env()?);

    tracing::info!("Connecting to database...");
    let mut opt = ConnectOptions::new(config.database_url.clone());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true);

    let db = Database::connect(opt).await?;

    tracing::info!("Running migrations...");
    tokimo_migration::Migrator::up(&db, None).await?;

    tracing::info!("Initializing storage...");
    let storage = tokimo_storage::storage_from_env().await?;

    let cache = Arc::new(infra::PgCache::new(db.clone()));
    let single_flight = Arc::new(infra::PgSingleFlight::new(Arc::new(db.clone())));
    let rate_limiter = Arc::new(infra::PgRateLimiter::new(db.clone()));
    let http = reqwest::Client::builder()
        .pool_max_idle_per_host(32)
        .pool_idle_timeout(Duration::from_secs(90))
        .tcp_keepalive(Duration::from_secs(60))
        .http2_keep_alive_interval(Some(Duration::from_secs(30)))
        .http2_keep_alive_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .user_agent("tokimo-server/0.1.0")
        .build()?;

    let state = AppState {
        db: db.clone(),
        storage,
        cache,
        single_flight,
        rate_limiter,
        http,
        config: config.clone(),
        metrics: Arc::new(MetricsStore::new()),
        provider_configs: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    };

    // Seed provider_configs with registry defaults; existing rows kept untouched.
    {
        use sea_orm::{sea_query::OnConflict, EntityTrait};
        for meta in tokimo_server::providers_registry::REGISTRY.iter() {
            let am = tokimo_server::db::entities::provider_configs::ActiveModel {
                key: sea_orm::Set(meta.key.to_string()),
                ttl_seconds: sea_orm::Set(meta.default_ttl_seconds as i32),
                enabled: sea_orm::Set(true),
                updated_at: sea_orm::Set(chrono::Utc::now().into()),
            };
            tokimo_server::db::entities::ProviderConfigs::insert(am)
                .on_conflict(
                    OnConflict::column(tokimo_server::db::entities::provider_configs::Column::Key)
                        .do_nothing()
                        .to_owned(),
                )
                .do_nothing()
                .exec(&db)
                .await?;
        }
    }
    state
        .reload_provider_configs()
        .await
        .map_err(|e| anyhow::anyhow!("failed to load provider_configs: {:?}", e))?;

    // Spawn sports prewarm task
    tokio::spawn(routes::sports::prewarm_task(state.clone()));

    // Spawn hot-search prewarm task (CDN-4): refresh all 19 sources every
    // SERVER_PREWARM_INTERVAL_SECS so user requests never cold-start.
    if config.prewarm_enabled {
        tracing::info!("Hot prewarm enabled (interval={}s)", config.prewarm_interval_secs);
        tokio::spawn(routes::hot::prewarm_task(state.clone()));
    } else {
        tracing::info!("Hot prewarm disabled (SERVER_PREWARM_ENABLED=false)");
    }

    if config.cache_cleanup_enabled {
        tracing::info!(
            interval_hours = config.cache_cleanup_interval_hours,
            "spawning cache_cleanup task"
        );
        tokimo_server::jobs::cache_cleanup::spawn(db.clone(), config.cache_cleanup_interval_hours);
    } else {
        tracing::info!("cache_cleanup disabled (SERVER_CACHE_CLEANUP_ENABLED=false)");
    }

    let cors = if config.cors_allowed_origins.is_empty() {
        CorsLayer::permissive()
    } else {
        let mut layer = CorsLayer::new();
        for origin in &config.cors_allowed_origins {
            let value: axum::http::HeaderValue = origin
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid CORS origin {}: {}", origin, e))?;
            layer = layer.allow_origin(value);
        }
        layer
            .allow_methods([Method::GET, Method::POST, Method::DELETE])
            .allow_headers(tower_http::cors::Any)
    };

    let app = Router::new()
        .nest("/api", routes::api_routes(state.clone()))
        .nest_service("/assets", ServeDir::new(&config.storage_local_root))
        .nest_service(
            "/admin",
            ServeDir::new("admin/dist").not_found_service(ServeFile::new("admin/dist/index.html")),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024));

    let listener = tokio::net::TcpListener::bind(&config.listen).await?;
    tracing::info!("Server listening on {}", config.listen);

    axum::serve(listener, app).await?;

    Ok(())
}

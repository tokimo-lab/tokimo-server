pub use sea_orm_migration::prelude::*;

mod m20250101_000001_create_service_keys;
mod m20250101_000002_create_provider_configs;
mod m20250101_000003_create_cache_entries;
mod m20250101_000004_create_rate_limit_buckets;
mod m20250101_000005_create_assets;
mod m20250101_000006_create_tmdb_movies;
mod m20250101_000007_create_tmdb_genres;
mod m20250101_000008_create_hot_search_items;
mod m20250101_000009_create_hot_search_snapshots;
mod m20250101_000010_create_sport_matches;
mod m20250101_000011_create_tmdb_objects;
mod m20250101_000012_create_tmdb_images;
mod m20250101_000013_create_omdb_titles;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_000001_create_service_keys::Migration),
            Box::new(m20250101_000002_create_provider_configs::Migration),
            Box::new(m20250101_000003_create_cache_entries::Migration),
            Box::new(m20250101_000004_create_rate_limit_buckets::Migration),
            Box::new(m20250101_000005_create_assets::Migration),
            Box::new(m20250101_000006_create_tmdb_movies::Migration),
            Box::new(m20250101_000007_create_tmdb_genres::Migration),
            Box::new(m20250101_000008_create_hot_search_items::Migration),
            Box::new(m20250101_000009_create_hot_search_snapshots::Migration),
            Box::new(m20250101_000010_create_sport_matches::Migration),
            Box::new(m20250101_000011_create_tmdb_objects::Migration),
            Box::new(m20250101_000012_create_tmdb_images::Migration),
            Box::new(m20250101_000013_create_omdb_titles::Migration),
        ]
    }
}

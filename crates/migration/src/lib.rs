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
mod m20250101_000014_create_thetvdb;
mod m20250101_000015_create_bangumi;
mod m20250101_000016_create_fanart;
mod m20250101_000017_create_douban;
mod m20250101_000018_create_spotify;
mod m20250101_000019_create_musicbrainz;
mod m20250101_000020_create_deezer;
mod m20250101_000021_create_lrclib;
mod m20250101_000022_create_qidian;
mod m20250101_000023_create_wikipedia;
mod m20250101_000030_create_openmeteo_forecasts;
mod m20250101_000031_create_nominatim_geocode;
mod m20250101_000032_create_holiday_years;
mod m20250101_000033_create_geocoding_results;
mod m20250101_000040_create_assrt;

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
            Box::new(m20250101_000014_create_thetvdb::Migration),
            Box::new(m20250101_000015_create_bangumi::Migration),
            Box::new(m20250101_000016_create_fanart::Migration),
            Box::new(m20250101_000017_create_douban::Migration),
            Box::new(m20250101_000018_create_spotify::Migration),
            Box::new(m20250101_000019_create_musicbrainz::Migration),
            Box::new(m20250101_000020_create_deezer::Migration),
            Box::new(m20250101_000021_create_lrclib::Migration),
            Box::new(m20250101_000030_create_openmeteo_forecasts::Migration),
            Box::new(m20250101_000031_create_nominatim_geocode::Migration),
            Box::new(m20250101_000022_create_qidian::Migration),
            Box::new(m20250101_000023_create_wikipedia::Migration),
            Box::new(m20250101_000032_create_holiday_years::Migration),
            Box::new(m20250101_000033_create_geocoding_results::Migration),
            Box::new(m20250101_000040_create_assrt::Migration),
        ]
    }
}

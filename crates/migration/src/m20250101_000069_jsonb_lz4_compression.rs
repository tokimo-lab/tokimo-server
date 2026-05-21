use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const JSONB_COMPRESSION_COLUMNS: &[(&str, &str)] = &[
    ("animetosho_cache", "raw_json"),
    ("assrt_searches", "raw_json"),
    ("assrt_sub_details", "raw_json"),
    ("bangumi_subjects", "raw_json"),
    ("bing_wallpaper_cache", "raw_json"),
    ("currency_rates", "raw_json"),
    ("deezer_albums", "raw_json"),
    ("deezer_artists", "raw_json"),
    ("deezer_tracks", "raw_json"),
    ("douban_subjects", "raw_json"),
    ("fanart_assets", "raw_json"),
    ("geocoding_results", "raw_json"),
    ("gestdown_cache", "raw_json"),
    ("github_releases", "raw_json"),
    ("hitokoto_cache", "raw_json"),
    ("holiday_years", "raw_json"),
    ("hot_search_snapshots", "payload"),
    ("itunes_cache", "raw_json"),
    ("lrclib_lyrics", "raw_json"),
    ("musicbrainz_artists", "raw_json"),
    ("musicbrainz_recordings", "raw_json"),
    ("musicbrainz_releases", "raw_json"),
    ("nominatim_geocode", "raw_json"),
    ("oauth_identities", "raw_profile"),
    ("omdb_titles", "raw_json"),
    ("openmeteo_forecasts", "raw_json"),
    ("opensubtitles_cache", "raw_json"),
    ("qidian_books", "raw_json"),
    ("regielive_cache", "raw_json"),
    ("service_keys", "scopes"),
    ("shooter_cache", "raw_json"),
    ("site_settings", "value"),
    ("sport_matches", "left_team"),
    ("sport_matches", "right_team"),
    ("spotify_albums", "raw_json"),
    ("spotify_artists", "raw_json"),
    ("spotify_tracks", "raw_json"),
    ("thetvdb_episodes", "raw_json"),
    ("thetvdb_series", "episodes_raw_json"),
    ("thetvdb_series", "raw_json"),
    ("tmdb_movies", "raw_json"),
    ("tmdb_objects", "raw_json"),
    ("wikipedia_summaries", "raw_json"),
    ("zenquotes_cache", "raw_json"),
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        set_compression(manager, "lz4").await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        set_compression(manager, "pglz").await
    }
}

async fn set_compression(manager: &SchemaManager<'_>, compression: &str) -> Result<(), DbErr> {
    let conn = manager.get_connection();
    for (table, column) in JSONB_COMPRESSION_COLUMNS {
        conn.execute_unprepared(&format!(
            "ALTER TABLE {table} ALTER COLUMN {column} SET COMPRESSION {compression}"
        ))
        .await?;
    }
    Ok(())
}

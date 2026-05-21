//! Retention-tier classification for cache tables.
//!
//! Each cache table is assigned a [`RetentionTier`] determining how long
//! data stays before being eligible for deletion by the cache_cleanup
//! background task. Permanent tables are intentionally absent from
//! [`CACHE_TABLES`] (they are never cleaned).

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionTier {
    Volatile, // 1 day
    Short,    // 7 days
    Medium,   // 30 days
    Permanent,
}

impl RetentionTier {
    pub const fn duration_secs(self) -> Option<i64> {
        match self {
            Self::Volatile => Some(86_400),
            Self::Short => Some(7 * 86_400),
            Self::Medium => Some(30 * 86_400),
            Self::Permanent => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CacheTableRetention {
    pub table: &'static str,
    pub timestamp_col: &'static str,
    pub tier: RetentionTier,
}

pub const CACHE_TABLES: &[CacheTableRetention] = &[
    // volatile — 1d
    CacheTableRetention {
        table: "hot_search_snapshots",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Volatile,
    },
    CacheTableRetention {
        table: "hot_search_items",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Volatile,
    },
    CacheTableRetention {
        table: "currency_rates",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Volatile,
    },
    CacheTableRetention {
        table: "openmeteo_forecasts",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Volatile,
    },
    CacheTableRetention {
        table: "zenquotes_cache",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Volatile,
    },
    CacheTableRetention {
        table: "hitokoto_cache",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Volatile,
    },
    CacheTableRetention {
        table: "bing_wallpaper_cache",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Volatile,
    },
    // short — 7d
    CacheTableRetention {
        table: "github_releases",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Short,
    },
    CacheTableRetention {
        table: "gestdown_cache",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Short,
    },
    CacheTableRetention {
        table: "regielive_cache",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Short,
    },
    CacheTableRetention {
        table: "shooter_cache",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Short,
    },
    CacheTableRetention {
        table: "animetosho_cache",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Short,
    },
    CacheTableRetention {
        table: "assrt_searches",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Short,
    },
    CacheTableRetention {
        table: "assrt_sub_details",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Short,
    },
    CacheTableRetention {
        table: "opensubtitles_cache",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Short,
    },
    CacheTableRetention {
        table: "lrclib_lyrics",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Short,
    },
    // medium — 30d
    CacheTableRetention {
        table: "sport_matches",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Medium,
    },
    CacheTableRetention {
        table: "holiday_years",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Medium,
    },
    CacheTableRetention {
        table: "geocoding_results",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Medium,
    },
    CacheTableRetention {
        table: "nominatim_geocode",
        timestamp_col: "fetched_at",
        tier: RetentionTier::Medium,
    },
    // Permanent (never cleaned, intentionally omitted):
    //   tmdb_movies, tmdb_genres, tmdb_images, tmdb_objects, omdb_titles,
    //   thetvdb_series, thetvdb_episodes, bangumi_subjects, fanart_assets,
    //   douban_subjects, spotify_albums/artists/tracks, deezer_albums/artists/tracks,
    //   musicbrainz_artists/recordings/releases, qidian_books,
    //   wikipedia_summaries, itunes_cache
];

// cache_entries gets a separate handler (expires_at column instead of fetched_at);
// the 10c-3 cleanup job will sweep it via `DELETE WHERE expires_at < now()`
// without consulting CACHE_TABLES.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_permanent_in_cache_tables() {
        for t in CACHE_TABLES {
            assert_ne!(
                t.tier,
                RetentionTier::Permanent,
                "table {} should not be permanent in CACHE_TABLES",
                t.table
            );
        }
    }

    #[test]
    fn durations_are_monotonic() {
        assert!(RetentionTier::Volatile.duration_secs().unwrap() < RetentionTier::Short.duration_secs().unwrap());
        assert!(RetentionTier::Short.duration_secs().unwrap() < RetentionTier::Medium.duration_secs().unwrap());
        assert_eq!(RetentionTier::Permanent.duration_secs(), None);
    }

    #[test]
    fn no_duplicate_tables() {
        let mut seen = std::collections::HashSet::new();
        for t in CACHE_TABLES {
            assert!(seen.insert(t.table), "duplicate table in CACHE_TABLES: {}", t.table);
        }
    }
}

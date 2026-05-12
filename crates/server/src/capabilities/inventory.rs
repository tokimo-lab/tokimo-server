//! Static inventory of all upstream providers proxied by tokimo-server.
//!
//! This is the source of truth for the public `/api/capabilities`
//! endpoint. Entries here mirror the routes registered in
//! [`crate::routes::api_routes`] and the metadata in
//! [`crate::providers_registry::REGISTRY`].
//!
//! Conventions:
//!
//! * `id` matches the route prefix (e.g. `"tmdb"` for `/api/tmdb/...`).
//! * `category` matches the high-level grouping in [`CATEGORIES`].
//! * `endpoints[].path` always starts with `/api/<id>/...`.
//! * `endpoints[].example` is a copy-pasteable curl path.
//! * `ai_hint` is 1-2 sentences in English: when to use this provider vs
//!   alternatives in the same category, plus any cross-provider lookup
//!   tricks (IMDb id, MBID, …).
//!
//! When you add a provider:
//! 1. add its `ProviderInfo` entry below,
//! 2. make sure the `id` is also present in one of [`CATEGORIES`],
//! 3. keep endpoints to the 1-3 most representative ones (overflow goes
//!    into `description` text — see `hot` below).

use serde::Serialize;

/// One endpoint a provider exposes.
#[derive(Debug, Clone, Serialize)]
pub struct EndpointInfo {
    pub method: &'static str,
    pub path: &'static str,
    pub example: &'static str,
    pub description: &'static str,
    /// Default cache TTL in seconds (mirrors
    /// [`crate::providers_registry::ProviderMeta::default_ttl_seconds`]).
    pub cache_ttl_seconds: i64,
}

/// Per-provider static metadata. Stats are joined in at request time.
#[derive(Debug, Clone, Serialize)]
pub struct ProviderInfo {
    pub id: &'static str,
    pub category: &'static str,
    pub summary: &'static str,
    pub upstream: &'static str,
    pub ai_hint: &'static str,
    pub endpoints: &'static [EndpointInfo],
    /// Only populated for the `hot` aggregator — list of accepted `?id=` values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_ids: Option<&'static [&'static str]>,
}

/// Category groupings used by the `/api/capabilities` response.
#[derive(Debug, Clone, Serialize)]
pub struct CategoryInfo {
    pub id: &'static str,
    pub providers: &'static [&'static str],
}

pub const CATEGORIES: &[CategoryInfo] = &[
    CategoryInfo {
        id: "movie-metadata",
        providers: &["tmdb", "omdb", "thetvdb", "douban", "fanart"],
    },
    CategoryInfo {
        id: "music-metadata",
        providers: &["spotify", "musicbrainz", "deezer", "itunes", "lrclib"],
    },
    CategoryInfo {
        id: "hot-search-aggregator",
        providers: &["hot"],
    },
    CategoryInfo {
        id: "subtitles",
        providers: &["opensubtitles", "assrt", "gestdown", "shooter", "regielive"],
    },
    CategoryInfo {
        id: "anime-metadata",
        providers: &["bangumi", "animetosho"],
    },
    CategoryInfo {
        id: "adult-metadata",
        providers: &["javbus", "javdb", "tpdb", "stashdb"],
    },
    CategoryInfo {
        id: "geo-weather",
        providers: &["geocoding", "nominatim", "openmeteo", "holiday"],
    },
    CategoryInfo {
        id: "misc-content",
        providers: &[
            "wikipedia",
            "qidian",
            "hitokoto",
            "zenquotes",
            "bing",
            "github",
            "currency",
            "sports",
        ],
    },
];

const HALF_DAY: i64 = 12 * 60 * 60;
const ONE_DAY: i64 = 24 * 60 * 60;
const SIX_HOURS: i64 = 6 * 60 * 60;
const ONE_HOUR: i64 = 60 * 60;
const HALF_HOUR: i64 = 30 * 60;
const FIVE_MIN: i64 = 5 * 60;

/// 19 hot-search source ids accepted by `/api/hot/list?id=<source>`.
/// Mirrors the `HotSource::id()` impls in `crates/providers/src/baidu_hot.rs`.
pub const HOT_SOURCE_IDS: &[&str] = &[
    "weibo",
    "bilibili",
    "baidu",
    "github",
    "hackernews",
    "v2ex",
    "toutiao",
    "36kr",
    "juejin",
    "sspai",
    "zhihu",
    "douyin",
    "douban-movie",
    "thepaper",
    "hupu",
    "ithome",
    "tieba",
    "linuxdo",
    "netease-news",
];

/// Full static inventory (34 providers).
pub const PROVIDERS: &[ProviderInfo] = &[
    // ----------------------------------------------------------------- movies
    ProviderInfo {
        id: "tmdb",
        category: "movie-metadata",
        summary: "TMDB 电影/剧集元数据代理（图片、详情、搜索、人物）",
        upstream: "api.themoviedb.org",
        ai_hint: "Cached ~12h. Best for posters and Chinese metadata. Use IMDb id via OMDb cross-lookup; image paths route through /api/tmdb/image/*.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/tmdb/movie/:id",
                example: "/api/tmdb/movie/550",
                description: "Fetch movie detail by TMDB id",
                cache_ttl_seconds: HALF_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/tmdb/tv/:id",
                example: "/api/tmdb/tv/1399",
                description: "Fetch TV show detail by TMDB id (also /season/:n and /season/:s/episode/:e)",
                cache_ttl_seconds: HALF_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/tmdb/image/*path",
                example: "/api/tmdb/image/w500/abc.jpg",
                description: "Proxied TMDB image CDN (forwards to image.tmdb.org with caching)",
                cache_ttl_seconds: HALF_DAY,
            },
        ],
        available_ids: None,
    },
    ProviderInfo {
        id: "omdb",
        category: "movie-metadata",
        summary: "OMDb 电影元数据代理（按 IMDb id 查询）",
        upstream: "www.omdbapi.com",
        ai_hint: "Use when you only have an IMDb id (ttXXXXXXX). Returns English-language metadata; combine with TMDB for posters/zh-CN titles.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/omdb/title/:imdb_id",
                example: "/api/omdb/title/tt1375666",
                description: "Lookup by IMDb id",
                cache_ttl_seconds: HALF_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/omdb/search",
                example: "/api/omdb/search?s=Inception",
                description: "Search by title (query param `s`)",
                cache_ttl_seconds: HALF_DAY,
            },
        ],
        available_ids: None,
    },
    ProviderInfo {
        id: "thetvdb",
        category: "movie-metadata",
        summary: "TheTVDB 剧集/单集元数据代理",
        upstream: "api4.thetvdb.com",
        ai_hint: "Authoritative episode-level metadata for TV series. Prefer over TMDB when you need accurate per-episode air dates / numbering.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/thetvdb/series/:id",
                example: "/api/thetvdb/series/121361",
                description: "Series detail by TheTVDB id",
                cache_ttl_seconds: HALF_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/thetvdb/series/:id/episodes",
                example: "/api/thetvdb/series/121361/episodes",
                description: "Full episode list for a series",
                cache_ttl_seconds: HALF_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/thetvdb/episode/:id",
                example: "/api/thetvdb/episode/3254641",
                description: "Single episode detail",
                cache_ttl_seconds: HALF_DAY,
            },
        ],
        available_ids: None,
    },
    ProviderInfo {
        id: "douban",
        category: "movie-metadata",
        summary: "豆瓣电影元数据 / 搜索代理",
        upstream: "movie.douban.com",
        ai_hint: "Best source for Chinese ratings, reviews and zh-CN titles. Geo-restricted upstream — always go through this proxy.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/douban/subject/:id",
                example: "/api/douban/subject/1292052",
                description: "豆瓣条目详情",
                cache_ttl_seconds: HALF_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/douban/search",
                example: "/api/douban/search?q=肖申克",
                description: "豆瓣电影搜索",
                cache_ttl_seconds: HALF_DAY,
            },
        ],
        available_ids: None,
    },
    ProviderInfo {
        id: "fanart",
        category: "movie-metadata",
        summary: "Fanart.tv 海报 / 艺术图代理",
        upstream: "webservice.fanart.tv",
        ai_hint: "High-quality clearart/logos/backgrounds keyed by TMDB or TVDB id. Use to enrich UI after you've already resolved the main id elsewhere.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/fanart/movie/:tmdb_id",
                example: "/api/fanart/movie/550",
                description: "Movie artwork by TMDB id",
                cache_ttl_seconds: ONE_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/fanart/tv/:tvdb_id",
                example: "/api/fanart/tv/121361",
                description: "TV artwork by TheTVDB id",
                cache_ttl_seconds: ONE_DAY,
            },
        ],
        available_ids: None,
    },
    // ------------------------------------------------------------------ music
    ProviderInfo {
        id: "spotify",
        category: "music-metadata",
        summary: "Spotify 元数据代理（艺人 / 专辑 / 单曲 / 搜索）",
        upstream: "api.spotify.com",
        ai_hint: "Read-only metadata. No playback streams. Use Spotify ids; for free-text lookups, prefer /search and then drill in.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/spotify/artist/:id",
                example: "/api/spotify/artist/4Z8W4fKeB5YxbusRsdQVPb",
                description: "Artist detail by Spotify id",
                cache_ttl_seconds: HALF_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/spotify/track/:id",
                example: "/api/spotify/track/7ouMYWpwJ422jRcDASZB7P",
                description: "Track detail by Spotify id (album/* also available)",
                cache_ttl_seconds: HALF_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/spotify/search",
                example: "/api/spotify/search?q=Radiohead&type=artist",
                description: "Full-text search; `type` ∈ {artist,album,track}",
                cache_ttl_seconds: ONE_HOUR,
            },
        ],
        available_ids: None,
    },
    ProviderInfo {
        id: "musicbrainz",
        category: "music-metadata",
        summary: "MusicBrainz 开放音乐元数据代理（MBID 查询）",
        upstream: "musicbrainz.org",
        ai_hint: "Use MBIDs (UUIDs). Best free source for cross-referenced artist/release/recording metadata when Spotify ids are unavailable.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/musicbrainz/artist/:mbid",
                example: "/api/musicbrainz/artist/a74b1b7f-71a5-4011-9441-d0b5e4122711",
                description: "Artist by MBID",
                cache_ttl_seconds: ONE_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/musicbrainz/release/:mbid",
                example: "/api/musicbrainz/release/0e8e8e8e-...",
                description: "Release by MBID (also /recording/:mbid)",
                cache_ttl_seconds: ONE_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/musicbrainz/search",
                example: "/api/musicbrainz/search?type=artist&query=radiohead",
                description: "Search by entity type",
                cache_ttl_seconds: ONE_HOUR,
            },
        ],
        available_ids: None,
    },
    ProviderInfo {
        id: "deezer",
        category: "music-metadata",
        summary: "Deezer 元数据代理",
        upstream: "api.deezer.com",
        ai_hint: "Alternative to Spotify with looser auth. Useful when Spotify rate-limits or for cover art at different sizes.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/deezer/track/:id",
                example: "/api/deezer/track/3135556",
                description: "Track detail (also /album/:id, /artist/:id)",
                cache_ttl_seconds: HALF_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/deezer/search",
                example: "/api/deezer/search?q=Daft+Punk",
                description: "Full-text search",
                cache_ttl_seconds: ONE_HOUR,
            },
        ],
        available_ids: None,
    },
    ProviderInfo {
        id: "itunes",
        category: "music-metadata",
        summary: "iTunes 专辑封面查询代理",
        upstream: "itunes.apple.com",
        ai_hint: "Single-purpose: fetch high-res album covers by artist+album text. No auth required upstream.",
        endpoints: &[EndpointInfo {
            method: "GET",
            path: "/api/itunes/album-cover",
            example: "/api/itunes/album-cover?artist=Daft+Punk&album=Discovery",
            description: "Resolve album cover URL by artist + album name",
            cache_ttl_seconds: ONE_DAY,
        }],
        available_ids: None,
    },
    ProviderInfo {
        id: "lrclib",
        category: "music-metadata",
        summary: "LRCLib 同步歌词查询代理",
        upstream: "lrclib.net",
        ai_hint: "Returns LRC-format synced lyrics. Use `/get` with exact track_name + artist_name; `/search` is fuzzy.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/lrclib/get",
                example: "/api/lrclib/get?track_name=Get+Lucky&artist_name=Daft+Punk",
                description: "Fetch synced lyrics by exact name",
                cache_ttl_seconds: ONE_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/lrclib/search",
                example: "/api/lrclib/search?q=get+lucky",
                description: "Fuzzy search",
                cache_ttl_seconds: ONE_HOUR,
            },
        ],
        available_ids: None,
    },
    // ----------------------------------------------------- hot search
    ProviderInfo {
        id: "hot",
        category: "hot-search-aggregator",
        summary: "19 个中文/技术站点的热榜聚合（微博/B站/知乎/V2EX/HN/...）",
        upstream: "mixed (per-source)",
        ai_hint: "Use /api/hot/list?id=<source>. 19 source ids accepted, see available_ids. Results pre-warmed every ~5 min server-side, so latency is cache-hit dominated.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/hot/sources",
                example: "/api/hot/sources",
                description: "List all available hot-search source ids and display names",
                cache_ttl_seconds: 0,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/hot/list",
                example: "/api/hot/list?id=weibo",
                description: "Fetch hot items for a single source id (see available_ids field)",
                cache_ttl_seconds: FIVE_MIN,
            },
        ],
        available_ids: Some(HOT_SOURCE_IDS),
    },
    // ----------------------------------------------------- subtitles
    ProviderInfo {
        id: "opensubtitles",
        category: "subtitles",
        summary: "OpenSubtitles 全球字幕搜索代理",
        upstream: "api.opensubtitles.com",
        ai_hint: "Largest cross-language subtitle DB. Use IMDb id as the most reliable key; falls back to text search.",
        endpoints: &[EndpointInfo {
            method: "GET",
            path: "/api/opensubtitles/search",
            example: "/api/opensubtitles/search?imdb_id=tt1375666&languages=zh-cn",
            description: "Subtitle search (imdb_id / query / languages params)",
            cache_ttl_seconds: SIX_HOURS,
        }],
        available_ids: None,
    },
    ProviderInfo {
        id: "assrt",
        category: "subtitles",
        summary: "射手网（Assrt）中文字幕搜索代理",
        upstream: "api.assrt.net",
        ai_hint: "Strong on zh-CN subtitles for movies and anime. Prefer over OpenSubtitles for Chinese content.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/assrt/search",
                example: "/api/assrt/search?q=肖申克",
                description: "Search subtitles by free-text query",
                cache_ttl_seconds: SIX_HOURS,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/assrt/sub/:id/detail",
                example: "/api/assrt/sub/123456/detail",
                description: "Subtitle detail + download links by Assrt id",
                cache_ttl_seconds: SIX_HOURS,
            },
        ],
        available_ids: None,
    },
    ProviderInfo {
        id: "gestdown",
        category: "subtitles",
        summary: "Gestdown (addic7ed 镜像) 字幕代理",
        upstream: "api.gestdown.info",
        ai_hint: "Best source for fan-translated TV-show subtitles. Resolve show id first via /shows/search.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/gestdown/shows/search",
                example: "/api/gestdown/shows/search?q=breaking+bad",
                description: "Resolve show id by name",
                cache_ttl_seconds: SIX_HOURS,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/gestdown/subtitles",
                example: "/api/gestdown/subtitles?show_id=...&season=1&episode=1",
                description: "Subtitle list per episode",
                cache_ttl_seconds: SIX_HOURS,
            },
        ],
        available_ids: None,
    },
    ProviderInfo {
        id: "shooter",
        category: "subtitles",
        summary: "射手 Shooter 字幕匹配代理",
        upstream: "www.shooter.cn",
        ai_hint: "Hash-based subtitle matching for local video files. Less coverage than assrt but no auth required.",
        endpoints: &[EndpointInfo {
            method: "GET",
            path: "/api/shooter/search",
            example: "/api/shooter/search?filehash=...&filename=movie.mkv",
            description: "Match subtitles by filehash + filename",
            cache_ttl_seconds: ONE_HOUR,
        }],
        available_ids: None,
    },
    ProviderInfo {
        id: "regielive",
        category: "subtitles",
        summary: "RegieLive 罗马尼亚语字幕代理",
        upstream: "subtitrari.regielive.ro",
        ai_hint: "Niche source — only useful if you need Romanian subtitles. Fall back to opensubtitles otherwise.",
        endpoints: &[EndpointInfo {
            method: "GET",
            path: "/api/regielive/search",
            example: "/api/regielive/search?q=inception",
            description: "Search Romanian subtitles by title",
            cache_ttl_seconds: SIX_HOURS,
        }],
        available_ids: None,
    },
    // ----------------------------------------------------- anime
    ProviderInfo {
        id: "bangumi",
        category: "anime-metadata",
        summary: "Bangumi.tv 动画 / 漫画 / 游戏元数据代理",
        upstream: "api.bgm.tv",
        ai_hint: "Authoritative for Japanese anime metadata in Chinese. Use /subject/:id once you know the bgm id; /search for discovery.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/bangumi/subject/:id",
                example: "/api/bangumi/subject/253",
                description: "Subject detail by bgm id",
                cache_ttl_seconds: HALF_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/bangumi/search",
                example: "/api/bangumi/search?q=进击的巨人",
                description: "Full-text search",
                cache_ttl_seconds: ONE_HOUR,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/bangumi/calendar",
                example: "/api/bangumi/calendar",
                description: "Current-season airing schedule (also /browse)",
                cache_ttl_seconds: ONE_HOUR,
            },
        ],
        available_ids: None,
    },
    ProviderInfo {
        id: "animetosho",
        category: "anime-metadata",
        summary: "AnimeTosho 种子 / 资源索引代理",
        upstream: "feed.animetosho.org",
        ai_hint: "Discoverability for anime release files (NOT a metadata source). Pair with bangumi for titles.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/animetosho/search",
                example: "/api/animetosho/search?q=frieren",
                description: "Search anime release entries",
                cache_ttl_seconds: HALF_HOUR,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/animetosho/torrent",
                example: "/api/animetosho/torrent?id=12345",
                description: "Torrent / file detail by id",
                cache_ttl_seconds: HALF_HOUR,
            },
        ],
        available_ids: None,
    },
    // ------------------------------------------------------ adult metadata
    ProviderInfo {
        id: "javbus",
        category: "adult-metadata",
        summary: "JavBus 影片元数据代理（成人内容，需 ADULT_MODE_ENABLED）",
        upstream: "javbus.com",
        ai_hint: "Gated by server-side ADULT_MODE flag — calls return 404 when disabled. Search by code.",
        endpoints: &[EndpointInfo {
            method: "GET",
            path: "/api/javbus/search",
            example: "/api/javbus/search?q=ABC-123",
            description: "Search by release code",
            cache_ttl_seconds: HALF_DAY,
        }],
        available_ids: None,
    },
    ProviderInfo {
        id: "javdb",
        category: "adult-metadata",
        summary: "JavDB 影片元数据代理（成人内容，需 ADULT_MODE_ENABLED）",
        upstream: "javdb.com",
        ai_hint: "Gated by ADULT_MODE flag. Higher metadata quality than javbus but stricter rate limits upstream.",
        endpoints: &[EndpointInfo {
            method: "GET",
            path: "/api/javdb/search",
            example: "/api/javdb/search?q=ABC-123",
            description: "Search by release code",
            cache_ttl_seconds: HALF_DAY,
        }],
        available_ids: None,
    },
    ProviderInfo {
        id: "tpdb",
        category: "adult-metadata",
        summary: "ThePornDB 元数据代理（成人内容，需 ADULT_MODE_ENABLED）",
        upstream: "api.theporndb.net",
        ai_hint: "Western-content counterpart to javdb. Gated by ADULT_MODE flag.",
        endpoints: &[EndpointInfo {
            method: "GET",
            path: "/api/tpdb/search",
            example: "/api/tpdb/search?q=keyword",
            description: "Search ThePornDB by free-text",
            cache_ttl_seconds: HALF_DAY,
        }],
        available_ids: None,
    },
    ProviderInfo {
        id: "stashdb",
        category: "adult-metadata",
        summary: "StashDB 元数据代理（成人内容，需 ADULT_MODE_ENABLED）",
        upstream: "stashdb.org",
        ai_hint: "Open metadata DB for adult scenes/performers. Gated by ADULT_MODE flag.",
        endpoints: &[EndpointInfo {
            method: "GET",
            path: "/api/stashdb/search",
            example: "/api/stashdb/search?q=keyword",
            description: "Search StashDB",
            cache_ttl_seconds: HALF_DAY,
        }],
        available_ids: None,
    },
    // ------------------------------------------------------ geo / weather
    ProviderInfo {
        id: "geocoding",
        category: "geo-weather",
        summary: "Open-Meteo 地名搜索 / 反查代理",
        upstream: "geocoding-api.open-meteo.com",
        ai_hint: "Lightweight geocoder. Forward = name→coords, reverse = coords→name. No auth.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/geocoding/forward",
                example: "/api/geocoding/forward?q=Tokyo",
                description: "Name → lat/lon",
                cache_ttl_seconds: ONE_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/geocoding/reverse",
                example: "/api/geocoding/reverse?lat=35.68&lon=139.69",
                description: "lat/lon → place name",
                cache_ttl_seconds: ONE_DAY,
            },
        ],
        available_ids: None,
    },
    ProviderInfo {
        id: "nominatim",
        category: "geo-weather",
        summary: "OSM Nominatim 地名搜索 / 反查代理",
        upstream: "nominatim.openstreetmap.org",
        ai_hint: "Higher-resolution OSM geocoder. Use when /geocoding misses small streets / POIs.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/nominatim/search",
                example: "/api/nominatim/search?q=Eiffel+Tower",
                description: "Free-text geocoder",
                cache_ttl_seconds: ONE_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/nominatim/reverse",
                example: "/api/nominatim/reverse?lat=48.85&lon=2.29",
                description: "Reverse geocode",
                cache_ttl_seconds: ONE_DAY,
            },
        ],
        available_ids: None,
    },
    ProviderInfo {
        id: "openmeteo",
        category: "geo-weather",
        summary: "Open-Meteo 天气 / 空气质量代理",
        upstream: "api.open-meteo.com",
        ai_hint: "Resolve coordinates via /api/geocoding/forward first, then pass to /forecast. No auth required upstream.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/openmeteo/forecast",
                example: "/api/openmeteo/forecast?latitude=35.68&longitude=139.69",
                description: "Weather forecast by coordinates",
                cache_ttl_seconds: HALF_HOUR,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/openmeteo/air-quality",
                example: "/api/openmeteo/air-quality?latitude=35.68&longitude=139.69",
                description: "Air-quality data by coordinates",
                cache_ttl_seconds: HALF_HOUR,
            },
        ],
        available_ids: None,
    },
    ProviderInfo {
        id: "holiday",
        category: "geo-weather",
        summary: "公共假期数据代理（按国家 + 年）",
        upstream: "date.nager.at",
        ai_hint: "Two-letter ISO country code + year. Stable, cache once per year.",
        endpoints: &[EndpointInfo {
            method: "GET",
            path: "/api/holiday/:country/:year",
            example: "/api/holiday/CN/2025",
            description: "Public holidays for a country/year",
            cache_ttl_seconds: ONE_DAY,
        }],
        available_ids: None,
    },
    // ------------------------------------------------------ misc content
    ProviderInfo {
        id: "wikipedia",
        category: "misc-content",
        summary: "Wikipedia 摘要查询代理",
        upstream: "*.wikipedia.org",
        ai_hint: "Returns 1-paragraph summary + thumbnail. Use the `lang` param to switch language editions.",
        endpoints: &[EndpointInfo {
            method: "GET",
            path: "/api/wikipedia/summary",
            example: "/api/wikipedia/summary?title=Albert_Einstein&lang=en",
            description: "Article summary by title",
            cache_ttl_seconds: ONE_DAY,
        }],
        available_ids: None,
    },
    ProviderInfo {
        id: "qidian",
        category: "misc-content",
        summary: "起点中文网图书元数据代理",
        upstream: "book.qidian.com",
        ai_hint: "Chinese web-novel catalog. Resolve book id via /search, then fetch /book/:id.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/qidian/book/:id",
                example: "/api/qidian/book/1010868264",
                description: "Book detail by qidian id",
                cache_ttl_seconds: HALF_DAY,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/qidian/search",
                example: "/api/qidian/search?q=诡秘之主",
                description: "Full-text search",
                cache_ttl_seconds: ONE_HOUR,
            },
        ],
        available_ids: None,
    },
    ProviderInfo {
        id: "hitokoto",
        category: "misc-content",
        summary: "一言（hitokoto）随机句子代理",
        upstream: "v1.hitokoto.cn",
        ai_hint: "Lightweight quote fountain. Output not cached aggressively — randomness expected.",
        endpoints: &[EndpointInfo {
            method: "GET",
            path: "/api/hitokoto/sentence",
            example: "/api/hitokoto/sentence",
            description: "Random Chinese quote / line",
            cache_ttl_seconds: 0,
        }],
        available_ids: None,
    },
    ProviderInfo {
        id: "zenquotes",
        category: "misc-content",
        summary: "ZenQuotes 英文随机名言代理",
        upstream: "zenquotes.io",
        ai_hint: "English-language counterpart to hitokoto. One quote per call.",
        endpoints: &[EndpointInfo {
            method: "GET",
            path: "/api/zenquotes/random",
            example: "/api/zenquotes/random",
            description: "Random English quote",
            cache_ttl_seconds: 0,
        }],
        available_ids: None,
    },
    ProviderInfo {
        id: "bing",
        category: "misc-content",
        summary: "Bing 每日壁纸代理",
        upstream: "www.bing.com",
        ai_hint: "Returns today's Bing wallpaper URL + metadata. Cached for ~1 day; safe to call from any client.",
        endpoints: &[EndpointInfo {
            method: "GET",
            path: "/api/bing/wallpaper",
            example: "/api/bing/wallpaper",
            description: "Today's Bing wallpaper",
            cache_ttl_seconds: ONE_DAY,
        }],
        available_ids: None,
    },
    ProviderInfo {
        id: "github",
        category: "misc-content",
        summary: "GitHub 仓库 releases 代理（按 owner/repo）",
        upstream: "api.github.com",
        ai_hint: "Use /latest for newest release, /list for full history. Counts against the server's GH token rate limit.",
        endpoints: &[
            EndpointInfo {
                method: "GET",
                path: "/api/github/releases/:owner/:repo/latest",
                example: "/api/github/releases/cli/cli/latest",
                description: "Latest GitHub release",
                cache_ttl_seconds: HALF_HOUR,
            },
            EndpointInfo {
                method: "GET",
                path: "/api/github/releases/:owner/:repo/list",
                example: "/api/github/releases/cli/cli/list",
                description: "Full release history",
                cache_ttl_seconds: HALF_HOUR,
            },
        ],
        available_ids: None,
    },
    ProviderInfo {
        id: "currency",
        category: "misc-content",
        summary: "汇率数据代理（多基础货币）",
        upstream: "open.er-api.com",
        ai_hint: "Daily-refreshed exchange rates. Returns rates relative to base currency in query.",
        endpoints: &[EndpointInfo {
            method: "GET",
            path: "/api/currency/rates",
            example: "/api/currency/rates?base=USD",
            description: "Exchange rates relative to `base`",
            cache_ttl_seconds: ONE_HOUR,
        }],
        available_ids: None,
    },
    ProviderInfo {
        id: "sports",
        category: "misc-content",
        summary: "赛程数据代理（按联赛/日期）",
        upstream: "various",
        ai_hint: "Sport schedule lookup. Filter via query params (date / league); coverage varies upstream.",
        endpoints: &[EndpointInfo {
            method: "GET",
            path: "/api/sports/schedule",
            example: "/api/sports/schedule?date=2025-01-01",
            description: "Sport schedule for a given date / league",
            cache_ttl_seconds: ONE_HOUR,
        }],
        available_ids: None,
    },
];

/// Lookup the static category id for a provider id, if known.
pub fn category_for(provider_id: &str) -> Option<&'static str> {
    PROVIDERS.iter().find(|p| p.id == provider_id).map(|p| p.category)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_provider_in_some_category() {
        for p in PROVIDERS {
            let in_categories = CATEGORIES.iter().any(|c| c.providers.contains(&p.id));
            assert!(in_categories, "provider {} missing from CATEGORIES", p.id);
        }
    }

    #[test]
    fn every_category_provider_exists() {
        for c in CATEGORIES {
            for pid in c.providers {
                assert!(
                    PROVIDERS.iter().any(|p| p.id == *pid),
                    "category {} references unknown provider {}",
                    c.id,
                    pid
                );
            }
        }
    }

    #[test]
    fn endpoint_paths_have_correct_prefix() {
        for p in PROVIDERS {
            for ep in p.endpoints {
                let expected = format!("/api/{}/", p.id);
                assert!(
                    ep.path.starts_with(&expected) || ep.path == format!("/api/{}", p.id),
                    "endpoint {} does not start with {}",
                    ep.path,
                    expected
                );
            }
        }
    }

    #[test]
    fn hot_has_19_source_ids() {
        assert_eq!(HOT_SOURCE_IDS.len(), 19);
    }

    #[test]
    fn provider_count_matches_categories() {
        let category_total: usize = CATEGORIES.iter().map(|c| c.providers.len()).sum();
        assert_eq!(category_total, PROVIDERS.len());
    }
}

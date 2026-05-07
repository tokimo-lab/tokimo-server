use argon2::PasswordHasher;
use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::Rng;
use sea_orm::{entity::*, PaginatorTrait, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};

use crate::{
    db::entities::{
        service_keys, AssrtSearches, AssrtSubDetails, BangumiSubjects, CacheEntries, DeezerAlbums, DeezerArtists,
        DeezerTracks, DoubanSubjects, FanartAssets, GeocodingResults, GithubReleases, HolidayYears, LrclibLyrics,
        MusicbrainzArtists, MusicbrainzRecordings, MusicbrainzReleases, NominatimGeocode, OmdbTitles,
        OpenmeteoForecasts, QidianBooks, ServiceKeys, SpotifyAlbums, SpotifyArtists, SpotifyTracks, ThetvdbEpisodes,
        ThetvdbSeries, TmdbImages, TmdbMovies, TmdbObjects, WikipediaSummaries,
    },
    middleware::AdminClaims,
    AppError, AppResult, AppState,
};

pub fn public_routes() -> Router<AppState> {
    Router::new().route("/login", post(login))
}

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/service-keys", get(list_keys).post(create_key).delete(delete_key))
        .route("/provider-configs", get(list_provider_configs))
        .route("/dashboard/overview", get(dashboard_overview))
        .route("/dashboard/timeseries", get(dashboard_timeseries))
        .route("/dashboard/by-provider", get(dashboard_by_provider))
        .route("/dashboard/recent-errors", get(dashboard_recent_errors))
        .route("/cache", get(list_cache))
        .route("/cache/tables", get(cache_tables))
        .route("/cache/:table", get(cache_list))
        .route("/cache/:table/:id", delete(cache_delete))
        .route("/cache/:table/:id/refresh", post(cache_refresh))
}

#[derive(Deserialize)]
struct LoginRequest {
    #[serde(alias = "key")]
    bootstrap_key: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

async fn login(State(state): State<AppState>, Json(req): Json<LoginRequest>) -> AppResult<Json<LoginResponse>> {
    if req.bootstrap_key != state.config.admin_bootstrap_key {
        return Err(AppError::Unauthorized);
    }

    let claims = AdminClaims {
        sub: "admin".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_ref()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))?;

    Ok(Json(LoginResponse { token }))
}

#[derive(Serialize)]
struct ServiceKeyResponse {
    id: uuid::Uuid,
    name: String,
    token_prefix: String,
    scopes: serde_json::Value,
    enabled: bool,
    expires_at: Option<String>,
    created_at: String,
    token: Option<String>,
}

async fn list_keys(State(state): State<AppState>) -> AppResult<Json<Vec<ServiceKeyResponse>>> {
    let keys = ServiceKeys::find()
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let response = keys
        .into_iter()
        .map(|k| ServiceKeyResponse {
            id: k.id,
            name: k.name,
            token_prefix: k.token_prefix,
            scopes: k.scopes,
            enabled: k.enabled,
            expires_at: k.expires_at.map(|dt| dt.to_string()),
            created_at: k.created_at.to_string(),
            token: None,
        })
        .collect();

    Ok(Json(response))
}

#[derive(Deserialize)]
struct CreateKeyRequest {
    name: String,
    scopes: Option<serde_json::Value>,
    expires_at: Option<String>,
}

async fn create_key(
    State(state): State<AppState>,
    Json(req): Json<CreateKeyRequest>,
) -> AppResult<Json<ServiceKeyResponse>> {
    let random_part: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let token = format!("tks_{}", random_part);
    let token_prefix = token[0..8].to_string();

    let hash = argon2::Argon2::default()
        .hash_password(
            token.as_bytes(),
            &argon2::password_hash::SaltString::generate(&mut rand::thread_rng()),
        )
        .map_err(|e| AppError::Internal(format!("Failed to hash token: {}", e)))?
        .to_string();

    let expires_at = if let Some(exp_str) = req.expires_at {
        Some(
            chrono::DateTime::parse_from_rfc3339(&exp_str)
                .map_err(|e| AppError::BadRequest(format!("Invalid expires_at: {}", e)))?,
        )
    } else {
        None
    };

    let model = service_keys::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        name: Set(req.name.clone()),
        token_hash: Set(hash),
        token_prefix: Set(token_prefix.clone()),
        scopes: Set(req.scopes.unwrap_or_else(|| serde_json::json!([]))),
        enabled: Set(true),
        expires_at: Set(expires_at),
        created_at: Set(chrono::Utc::now().into()),
    };

    let result = ServiceKeys::insert(model)
        .exec_with_returning(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(ServiceKeyResponse {
        id: result.id,
        name: result.name,
        token_prefix: result.token_prefix,
        scopes: result.scopes,
        enabled: result.enabled,
        expires_at: result.expires_at.map(|dt| dt.to_string()),
        created_at: result.created_at.to_string(),
        token: Some(token),
    }))
}

#[derive(Deserialize)]
struct DeleteKeyRequest {
    id: uuid::Uuid,
}

async fn delete_key(
    State(state): State<AppState>,
    Json(req): Json<DeleteKeyRequest>,
) -> AppResult<Json<serde_json::Value>> {
    ServiceKeys::delete_by_id(req.id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

async fn list_provider_configs(State(_state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "configs": [] })))
}

async fn list_cache(State(_state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "entries": [] })))
}

#[derive(Serialize)]
struct DashboardOverviewResponse {
    total_keys: u64,
    total_providers: u64,
    cache_entries_total: u64,
    calls_24h: u64,
    errors_24h: u64,
    cache_hit_ratio_24h: f64,
}

async fn dashboard_overview(State(state): State<AppState>) -> AppResult<Json<DashboardOverviewResponse>> {
    let total_service_keys = ServiceKeys::find()
        .count(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let total_cache_entries = CacheEntries::find()
        .count(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let overview = state.metrics.overview_stats_24h();

    Ok(Json(DashboardOverviewResponse {
        total_keys: total_service_keys,
        total_providers: 20,
        cache_entries_total: total_cache_entries,
        calls_24h: overview.calls_24h,
        errors_24h: overview.errors_24h,
        cache_hit_ratio_24h: overview.hit_ratio_24h,
    }))
}

#[derive(Deserialize)]
struct TimeseriesQuery {
    range: Option<String>,
    bucket: Option<String>,
}

async fn dashboard_timeseries(
    State(state): State<AppState>,
    Query(query): Query<TimeseriesQuery>,
) -> AppResult<Json<Vec<crate::metrics::TimeseriesBucket>>> {
    let range_secs = parse_duration_secs(query.range.as_deref().unwrap_or("24h"))?;
    let bucket_secs = parse_duration_secs(query.bucket.as_deref().unwrap_or("1h"))?;
    Ok(Json(state.metrics.query_timeseries(range_secs, bucket_secs)))
}

#[derive(Deserialize)]
struct ByProviderQuery {
    range: Option<String>,
}

async fn dashboard_by_provider(
    State(state): State<AppState>,
    Query(query): Query<ByProviderQuery>,
) -> AppResult<Json<Vec<crate::metrics::ProviderStats>>> {
    let range_secs = parse_duration_secs(query.range.as_deref().unwrap_or("24h"))?;
    Ok(Json(state.metrics.query_by_provider(range_secs)))
}

#[derive(Deserialize)]
struct RecentErrorsQuery {
    limit: Option<usize>,
}

async fn dashboard_recent_errors(
    State(state): State<AppState>,
    Query(query): Query<RecentErrorsQuery>,
) -> AppResult<Json<Vec<crate::metrics::ErrorSample>>> {
    let limit = query.limit.unwrap_or(20).min(100);
    Ok(Json(state.metrics.query_recent_errors(limit)))
}

fn parse_duration_secs(value: &str) -> Result<i64, AppError> {
    if value.len() < 2 {
        return Err(AppError::BadRequest(format!("Invalid duration: {}", value)));
    }

    let (number, suffix) = value.split_at(value.len() - 1);
    let multiplier = match suffix {
        "m" => 60,
        "h" => 60 * 60,
        "d" => 24 * 60 * 60,
        _ => return Err(AppError::BadRequest(format!("Invalid duration suffix: {}", value))),
    };
    let amount = number
        .parse::<i64>()
        .map_err(|_| AppError::BadRequest(format!("Invalid duration: {}", value)))?;
    if amount <= 0 {
        return Err(AppError::BadRequest(format!("Duration must be positive: {}", value)));
    }
    amount
        .checked_mul(multiplier)
        .ok_or_else(|| AppError::BadRequest(format!("Duration is too large: {}", value)))
}

struct CacheTableInfo {
    name: &'static str,
    pk_cols: &'static [&'static str],
    has_raw_json: bool,
}

const CACHE_TABLES: &[CacheTableInfo] = &[
    CacheTableInfo {
        name: "assrt_searches",
        pk_cols: &["cache_key"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "assrt_sub_details",
        pk_cols: &["sub_id"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "bangumi_subjects",
        pk_cols: &["id"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "deezer_albums",
        pk_cols: &["deezer_id"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "deezer_artists",
        pk_cols: &["deezer_id"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "deezer_tracks",
        pk_cols: &["deezer_id"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "douban_subjects",
        pk_cols: &["douban_id"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "fanart_assets",
        pk_cols: &["kind", "foreign_id"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "geocoding_results",
        pk_cols: &["cache_key"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "github_releases",
        pk_cols: &["cache_key"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "holiday_years",
        pk_cols: &["country", "year"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "lrclib_lyrics",
        pk_cols: &["cache_key"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "musicbrainz_artists",
        pk_cols: &["mbid"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "musicbrainz_recordings",
        pk_cols: &["mbid"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "musicbrainz_releases",
        pk_cols: &["mbid"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "nominatim_geocode",
        pk_cols: &["cache_key"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "omdb_titles",
        pk_cols: &["imdb_id"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "openmeteo_forecasts",
        pk_cols: &["cache_key"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "qidian_books",
        pk_cols: &["qidian_id"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "spotify_albums",
        pk_cols: &["spotify_id"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "spotify_artists",
        pk_cols: &["spotify_id"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "spotify_tracks",
        pk_cols: &["spotify_id"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "thetvdb_episodes",
        pk_cols: &["id"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "thetvdb_series",
        pk_cols: &["id"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "tmdb_images",
        pk_cols: &["image_path"],
        has_raw_json: false,
    },
    CacheTableInfo {
        name: "tmdb_movies",
        pk_cols: &["id"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "tmdb_objects",
        pk_cols: &["kind", "key"],
        has_raw_json: true,
    },
    CacheTableInfo {
        name: "wikipedia_summaries",
        pk_cols: &["cache_key"],
        has_raw_json: true,
    },
];

#[derive(Serialize)]
struct CacheTableResponse {
    name: &'static str,
    row_count: u64,
    avg_ttl_remaining_seconds: Option<i64>,
}

async fn cache_tables(State(state): State<AppState>) -> AppResult<Json<Vec<CacheTableResponse>>> {
    macro_rules! count_table {
        ($entity:ty) => {
            <$entity>::find()
                .count(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
        };
    }

    let mut tables = Vec::with_capacity(CACHE_TABLES.len());
    for table in CACHE_TABLES {
        let row_count = match table.name {
            "assrt_searches" => count_table!(AssrtSearches),
            "assrt_sub_details" => count_table!(AssrtSubDetails),
            "bangumi_subjects" => count_table!(BangumiSubjects),
            "deezer_albums" => count_table!(DeezerAlbums),
            "deezer_artists" => count_table!(DeezerArtists),
            "deezer_tracks" => count_table!(DeezerTracks),
            "douban_subjects" => count_table!(DoubanSubjects),
            "fanart_assets" => count_table!(FanartAssets),
            "geocoding_results" => count_table!(GeocodingResults),
            "github_releases" => count_table!(GithubReleases),
            "holiday_years" => count_table!(HolidayYears),
            "lrclib_lyrics" => count_table!(LrclibLyrics),
            "musicbrainz_artists" => count_table!(MusicbrainzArtists),
            "musicbrainz_recordings" => count_table!(MusicbrainzRecordings),
            "musicbrainz_releases" => count_table!(MusicbrainzReleases),
            "nominatim_geocode" => count_table!(NominatimGeocode),
            "omdb_titles" => count_table!(OmdbTitles),
            "openmeteo_forecasts" => count_table!(OpenmeteoForecasts),
            "qidian_books" => count_table!(QidianBooks),
            "spotify_albums" => count_table!(SpotifyAlbums),
            "spotify_artists" => count_table!(SpotifyArtists),
            "spotify_tracks" => count_table!(SpotifyTracks),
            "thetvdb_episodes" => count_table!(ThetvdbEpisodes),
            "thetvdb_series" => count_table!(ThetvdbSeries),
            "tmdb_images" => count_table!(TmdbImages),
            "tmdb_movies" => count_table!(TmdbMovies),
            "tmdb_objects" => count_table!(TmdbObjects),
            "wikipedia_summaries" => count_table!(WikipediaSummaries),
            _ => return Err(AppError::BadRequest(format!("Unknown cache table: {}", table.name))),
        };
        tables.push(CacheTableResponse {
            name: table.name,
            row_count,
            avg_ttl_remaining_seconds: None,
        });
    }

    Ok(Json(tables))
}

#[derive(Deserialize)]
struct CacheListQuery {
    limit: Option<u64>,
    offset: Option<u64>,
}

#[derive(Serialize)]
struct CacheRowResponse {
    id: String,
    key: String,
    fetched_at: String,
    raw_preview: Option<String>,
}

#[derive(Serialize)]
struct CacheListResponse {
    table: String,
    limit: u64,
    offset: u64,
    rows: Vec<CacheRowResponse>,
}

async fn cache_list(
    State(state): State<AppState>,
    Path(table): Path<String>,
    Query(query): Query<CacheListQuery>,
) -> AppResult<Json<CacheListResponse>> {
    let table_info = cache_table_info(&table)?;
    let limit = query.limit.unwrap_or(50).min(200);
    let offset = query.offset.unwrap_or(0);
    let rows = cache_rows_as_json(&state, table_info, limit, offset).await?;
    let rows = rows
        .into_iter()
        .map(|row| cache_row_response(row, table_info))
        .collect();

    Ok(Json(CacheListResponse {
        table,
        limit,
        offset,
        rows,
    }))
}

async fn cache_rows_as_json(
    state: &AppState,
    table: &CacheTableInfo,
    limit: u64,
    offset: u64,
) -> AppResult<Vec<serde_json::Value>> {
    macro_rules! list_entity {
        ($entity:ty, $fetched_at:expr) => {{
            <$entity>::find()
                .order_by_desc($fetched_at)
                .limit(limit)
                .offset(offset)
                .into_json()
                .all(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
        }};
    }

    let rows = match table.name {
        "assrt_searches" => list_entity!(AssrtSearches, crate::db::entities::assrt_searches::Column::FetchedAt),
        "assrt_sub_details" => list_entity!(
            AssrtSubDetails,
            crate::db::entities::assrt_sub_details::Column::FetchedAt
        ),
        "bangumi_subjects" => list_entity!(
            BangumiSubjects,
            crate::db::entities::bangumi_subjects::Column::FetchedAt
        ),
        "deezer_albums" => list_entity!(DeezerAlbums, crate::db::entities::deezer_albums::Column::FetchedAt),
        "deezer_artists" => list_entity!(DeezerArtists, crate::db::entities::deezer_artists::Column::FetchedAt),
        "deezer_tracks" => list_entity!(DeezerTracks, crate::db::entities::deezer_tracks::Column::FetchedAt),
        "douban_subjects" => list_entity!(DoubanSubjects, crate::db::entities::douban_subjects::Column::FetchedAt),
        "fanart_assets" => list_entity!(FanartAssets, crate::db::entities::fanart_assets::Column::FetchedAt),
        "geocoding_results" => list_entity!(
            GeocodingResults,
            crate::db::entities::geocoding_results::Column::FetchedAt
        ),
        "github_releases" => list_entity!(GithubReleases, crate::db::entities::github_releases::Column::FetchedAt),
        "holiday_years" => list_entity!(HolidayYears, crate::db::entities::holiday_years::Column::FetchedAt),
        "lrclib_lyrics" => list_entity!(LrclibLyrics, crate::db::entities::lrclib_lyrics::Column::FetchedAt),
        "musicbrainz_artists" => list_entity!(
            MusicbrainzArtists,
            crate::db::entities::musicbrainz_artists::Column::FetchedAt
        ),
        "musicbrainz_recordings" => {
            list_entity!(
                MusicbrainzRecordings,
                crate::db::entities::musicbrainz_recordings::Column::FetchedAt
            )
        }
        "musicbrainz_releases" => list_entity!(
            MusicbrainzReleases,
            crate::db::entities::musicbrainz_releases::Column::FetchedAt
        ),
        "nominatim_geocode" => list_entity!(
            NominatimGeocode,
            crate::db::entities::nominatim_geocode::Column::FetchedAt
        ),
        "omdb_titles" => list_entity!(OmdbTitles, crate::db::entities::omdb_titles::Column::FetchedAt),
        "openmeteo_forecasts" => list_entity!(
            OpenmeteoForecasts,
            crate::db::entities::openmeteo_forecasts::Column::FetchedAt
        ),
        "qidian_books" => list_entity!(QidianBooks, crate::db::entities::qidian_books::Column::FetchedAt),
        "spotify_albums" => list_entity!(SpotifyAlbums, crate::db::entities::spotify_albums::Column::FetchedAt),
        "spotify_artists" => list_entity!(SpotifyArtists, crate::db::entities::spotify_artists::Column::FetchedAt),
        "spotify_tracks" => list_entity!(SpotifyTracks, crate::db::entities::spotify_tracks::Column::FetchedAt),
        "thetvdb_episodes" => list_entity!(
            ThetvdbEpisodes,
            crate::db::entities::thetvdb_episodes::Column::FetchedAt
        ),
        "thetvdb_series" => list_entity!(ThetvdbSeries, crate::db::entities::thetvdb_series::Column::FetchedAt),
        "tmdb_images" => list_entity!(TmdbImages, crate::db::entities::tmdb_images::Column::FetchedAt),
        "tmdb_movies" => list_entity!(TmdbMovies, crate::db::entities::tmdb_movies::Column::FetchedAt),
        "tmdb_objects" => list_entity!(TmdbObjects, crate::db::entities::tmdb_objects::Column::FetchedAt),
        "wikipedia_summaries" => list_entity!(
            WikipediaSummaries,
            crate::db::entities::wikipedia_summaries::Column::FetchedAt
        ),
        _ => return Err(AppError::BadRequest(format!("Unknown cache table: {}", table.name))),
    };

    Ok(rows)
}

async fn cache_delete(
    State(state): State<AppState>,
    Path((table, id)): Path<(String, String)>,
) -> AppResult<Json<serde_json::Value>> {
    let table_info = cache_table_info(&table)?;
    let pv = parse_pk_values(table_info, &id)?;

    macro_rules! del_str {
        ($entity:ty) => {{
            <$entity>::delete_by_id(pv[0].clone())
                .exec(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .rows_affected
        }};
    }
    macro_rules! del_i64 {
        ($entity:ty) => {{
            let n: i64 = pv[0]
                .parse()
                .map_err(|_| AppError::BadRequest(format!("Invalid id: {}", pv[0])))?;
            <$entity>::delete_by_id(n)
                .exec(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .rows_affected
        }};
    }
    macro_rules! del_i32 {
        ($entity:ty) => {{
            let n: i32 = pv[0]
                .parse()
                .map_err(|_| AppError::BadRequest(format!("Invalid id: {}", pv[0])))?;
            <$entity>::delete_by_id(n)
                .exec(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .rows_affected
        }};
    }

    let rows_affected = match table_info.name {
        "assrt_searches" => del_str!(AssrtSearches),
        "assrt_sub_details" => del_str!(AssrtSubDetails),
        "bangumi_subjects" => del_i64!(BangumiSubjects),
        "deezer_albums" => del_i64!(DeezerAlbums),
        "deezer_artists" => del_i64!(DeezerArtists),
        "deezer_tracks" => del_i64!(DeezerTracks),
        "douban_subjects" => del_str!(DoubanSubjects),
        "fanart_assets" => {
            let foreign_id: i64 = pv[1]
                .parse()
                .map_err(|_| AppError::BadRequest(format!("Invalid id: {}", pv[1])))?;
            FanartAssets::delete_by_id((pv[0].clone(), foreign_id))
                .exec(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .rows_affected
        }
        "geocoding_results" => del_str!(GeocodingResults),
        "github_releases" => del_str!(GithubReleases),
        "holiday_years" => {
            let year: i32 = pv[1]
                .parse()
                .map_err(|_| AppError::BadRequest(format!("Invalid id: {}", pv[1])))?;
            HolidayYears::delete_by_id((pv[0].clone(), year))
                .exec(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .rows_affected
        }
        "lrclib_lyrics" => del_str!(LrclibLyrics),
        "musicbrainz_artists" => del_str!(MusicbrainzArtists),
        "musicbrainz_recordings" => del_str!(MusicbrainzRecordings),
        "musicbrainz_releases" => del_str!(MusicbrainzReleases),
        "nominatim_geocode" => del_str!(NominatimGeocode),
        "omdb_titles" => del_str!(OmdbTitles),
        "openmeteo_forecasts" => del_str!(OpenmeteoForecasts),
        "qidian_books" => del_str!(QidianBooks),
        "spotify_albums" => del_str!(SpotifyAlbums),
        "spotify_artists" => del_str!(SpotifyArtists),
        "spotify_tracks" => del_str!(SpotifyTracks),
        "thetvdb_episodes" => del_i64!(ThetvdbEpisodes),
        "thetvdb_series" => del_i64!(ThetvdbSeries),
        "tmdb_images" => del_str!(TmdbImages),
        "tmdb_movies" => del_i32!(TmdbMovies),
        "tmdb_objects" => {
            TmdbObjects::delete_by_id((pv[0].clone(), pv[1].clone()))
                .exec(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .rows_affected
        }
        "wikipedia_summaries" => del_str!(WikipediaSummaries),
        _ => {
            return Err(AppError::BadRequest(format!(
                "Unknown cache table: {}",
                table_info.name
            )))
        }
    };

    if rows_affected == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(
        serde_json::json!({ "success": true, "rows_affected": rows_affected }),
    ))
}

fn epoch_start() -> chrono::DateTime<chrono::FixedOffset> {
    use chrono::{FixedOffset, TimeZone};
    FixedOffset::east_opt(0).unwrap().timestamp_opt(0, 0).unwrap()
}

async fn cache_refresh(
    State(state): State<AppState>,
    Path((table, id)): Path<(String, String)>,
) -> AppResult<Json<serde_json::Value>> {
    use crate::db::entities::{
        assrt_searches, assrt_sub_details, bangumi_subjects, deezer_albums, deezer_artists, deezer_tracks,
        douban_subjects, fanart_assets, geocoding_results, github_releases, holiday_years, lrclib_lyrics,
        musicbrainz_artists, musicbrainz_recordings, musicbrainz_releases, nominatim_geocode, omdb_titles,
        openmeteo_forecasts, qidian_books, spotify_albums, spotify_artists, spotify_tracks, thetvdb_episodes,
        thetvdb_series, tmdb_images, tmdb_movies, tmdb_objects, wikipedia_summaries,
    };

    let table_info = cache_table_info(&table)?;
    let pv = parse_pk_values(table_info, &id)?;
    let epoch = epoch_start();

    macro_rules! refresh_str {
        ($entity:ty, $active:ty) => {{
            let model = <$entity>::find_by_id(pv[0].clone())
                .one(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .ok_or(AppError::NotFound)?;
            let mut am: $active = model.into();
            am.fetched_at = Set(epoch);
            am.update(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }};
    }
    macro_rules! refresh_i64 {
        ($entity:ty, $active:ty) => {{
            let n: i64 = pv[0]
                .parse()
                .map_err(|_| AppError::BadRequest(format!("Invalid id: {}", pv[0])))?;
            let model = <$entity>::find_by_id(n)
                .one(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .ok_or(AppError::NotFound)?;
            let mut am: $active = model.into();
            am.fetched_at = Set(epoch);
            am.update(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }};
    }
    macro_rules! refresh_i32 {
        ($entity:ty, $active:ty) => {{
            let n: i32 = pv[0]
                .parse()
                .map_err(|_| AppError::BadRequest(format!("Invalid id: {}", pv[0])))?;
            let model = <$entity>::find_by_id(n)
                .one(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .ok_or(AppError::NotFound)?;
            let mut am: $active = model.into();
            am.fetched_at = Set(epoch);
            am.update(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }};
    }

    match table_info.name {
        "assrt_searches" => refresh_str!(AssrtSearches, assrt_searches::ActiveModel),
        "assrt_sub_details" => refresh_str!(AssrtSubDetails, assrt_sub_details::ActiveModel),
        "bangumi_subjects" => refresh_i64!(BangumiSubjects, bangumi_subjects::ActiveModel),
        "deezer_albums" => refresh_i64!(DeezerAlbums, deezer_albums::ActiveModel),
        "deezer_artists" => refresh_i64!(DeezerArtists, deezer_artists::ActiveModel),
        "deezer_tracks" => refresh_i64!(DeezerTracks, deezer_tracks::ActiveModel),
        "douban_subjects" => refresh_str!(DoubanSubjects, douban_subjects::ActiveModel),
        "fanart_assets" => {
            let foreign_id: i64 = pv[1]
                .parse()
                .map_err(|_| AppError::BadRequest(format!("Invalid id: {}", pv[1])))?;
            let model = FanartAssets::find_by_id((pv[0].clone(), foreign_id))
                .one(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .ok_or(AppError::NotFound)?;
            let mut am: fanart_assets::ActiveModel = model.into();
            am.fetched_at = Set(epoch);
            am.update(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }
        "geocoding_results" => refresh_str!(GeocodingResults, geocoding_results::ActiveModel),
        "github_releases" => refresh_str!(GithubReleases, github_releases::ActiveModel),
        "holiday_years" => {
            let year: i32 = pv[1]
                .parse()
                .map_err(|_| AppError::BadRequest(format!("Invalid id: {}", pv[1])))?;
            let model = HolidayYears::find_by_id((pv[0].clone(), year))
                .one(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .ok_or(AppError::NotFound)?;
            let mut am: holiday_years::ActiveModel = model.into();
            am.fetched_at = Set(epoch);
            am.update(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }
        "lrclib_lyrics" => refresh_str!(LrclibLyrics, lrclib_lyrics::ActiveModel),
        "musicbrainz_artists" => refresh_str!(MusicbrainzArtists, musicbrainz_artists::ActiveModel),
        "musicbrainz_recordings" => refresh_str!(MusicbrainzRecordings, musicbrainz_recordings::ActiveModel),
        "musicbrainz_releases" => refresh_str!(MusicbrainzReleases, musicbrainz_releases::ActiveModel),
        "nominatim_geocode" => refresh_str!(NominatimGeocode, nominatim_geocode::ActiveModel),
        "omdb_titles" => refresh_str!(OmdbTitles, omdb_titles::ActiveModel),
        "openmeteo_forecasts" => refresh_str!(OpenmeteoForecasts, openmeteo_forecasts::ActiveModel),
        "qidian_books" => refresh_str!(QidianBooks, qidian_books::ActiveModel),
        "spotify_albums" => refresh_str!(SpotifyAlbums, spotify_albums::ActiveModel),
        "spotify_artists" => refresh_str!(SpotifyArtists, spotify_artists::ActiveModel),
        "spotify_tracks" => refresh_str!(SpotifyTracks, spotify_tracks::ActiveModel),
        "thetvdb_episodes" => refresh_i64!(ThetvdbEpisodes, thetvdb_episodes::ActiveModel),
        "thetvdb_series" => refresh_i64!(ThetvdbSeries, thetvdb_series::ActiveModel),
        "tmdb_images" => refresh_str!(TmdbImages, tmdb_images::ActiveModel),
        "tmdb_movies" => refresh_i32!(TmdbMovies, tmdb_movies::ActiveModel),
        "tmdb_objects" => {
            let model = TmdbObjects::find_by_id((pv[0].clone(), pv[1].clone()))
                .one(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .ok_or(AppError::NotFound)?;
            let mut am: tmdb_objects::ActiveModel = model.into();
            am.fetched_at = Set(epoch);
            am.update(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }
        "wikipedia_summaries" => refresh_str!(WikipediaSummaries, wikipedia_summaries::ActiveModel),
        _ => {
            return Err(AppError::BadRequest(format!(
                "Unknown cache table: {}",
                table_info.name
            )))
        }
    }

    Ok(Json(serde_json::json!({ "success": true, "rows_affected": 1 })))
}

fn cache_table_info(name: &str) -> AppResult<&'static CacheTableInfo> {
    CACHE_TABLES
        .iter()
        .find(|table| table.name == name)
        .ok_or_else(|| AppError::BadRequest(format!("Unknown cache table: {}", name)))
}

fn cache_row_response(row: serde_json::Value, table: &CacheTableInfo) -> CacheRowResponse {
    let id = table
        .pk_cols
        .iter()
        .map(|col| json_value_to_key(row.get(*col)))
        .collect::<Vec<_>>()
        .join("|");
    let fetched_at = row
        .get("fetched_at")
        .and_then(|value| value.as_str())
        .map(str::to_owned)
        .unwrap_or_default();
    let preview_source = if table.has_raw_json {
        row.get("raw_json").unwrap_or(&row)
    } else {
        &row
    };

    CacheRowResponse {
        id: id.clone(),
        key: id,
        fetched_at,
        raw_preview: Some(truncate_preview(&preview_source.to_string())),
    }
}

fn json_value_to_key(value: Option<&serde_json::Value>) -> String {
    match value {
        Some(serde_json::Value::String(value)) => value.clone(),
        Some(value) => value.to_string(),
        None => String::new(),
    }
}

fn truncate_preview(value: &str) -> String {
    const MAX_LEN: usize = 200;
    if value.len() <= MAX_LEN {
        return value.to_owned();
    }

    match value.char_indices().nth(MAX_LEN) {
        Some((index, _)) => format!("{}...", &value[..index]),
        None => value.to_owned(),
    }
}

fn parse_pk_values(table: &CacheTableInfo, id: &str) -> AppResult<Vec<String>> {
    let values = id.split('|').map(str::to_owned).collect::<Vec<_>>();
    if values.len() != table.pk_cols.len() {
        return Err(AppError::BadRequest(format!(
            "Invalid id for {}: expected {} part(s)",
            table.name,
            table.pk_cols.len()
        )));
    }
    Ok(values)
}

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_providers::nominatim;

use crate::{
    db::entities::{nominatim_geocode, NominatimGeocode},
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/search", get(get_search))
        .route("/reverse", get(get_reverse))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: u8,
    #[serde(default = "default_lang")]
    pub lang: String,
}

#[derive(Deserialize)]
pub struct ReverseQuery {
    pub lat: f64,
    pub lon: f64,
    #[serde(default = "default_lang")]
    pub lang: String,
}

fn default_limit() -> u8 {
    5
}

fn default_lang() -> String {
    "en".to_string()
}

async fn get_search(State(state): State<AppState>, Query(q): Query<SearchQuery>) -> AppResult<Json<serde_json::Value>> {
    let limit_str = q.limit.to_string();
    let key = nominatim::cache_key("search", &[("q", &q.q), ("limit", &limit_str), ("lang", &q.lang)]);
    fetch_or_cache(state, key, "nominatim:search", move |http, ua| {
        let q_owned = q.q.clone();
        let lang = q.lang.clone();
        let limit = q.limit;
        async move { nominatim::search(&http, &ua, &q_owned, limit, &lang).await }
    })
    .await
}

async fn get_reverse(
    State(state): State<AppState>,
    Query(q): Query<ReverseQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let key = nominatim::cache_key(
        "reverse",
        &[
            ("lat", &format!("{:.6}", q.lat)),
            ("lon", &format!("{:.6}", q.lon)),
            ("lang", &q.lang),
        ],
    );
    fetch_or_cache(state, key, "nominatim:reverse", move |http, ua| {
        let lang = q.lang.clone();
        let lat = q.lat;
        let lon = q.lon;
        async move { nominatim::reverse(&http, &ua, lat, lon, &lang).await }
    })
    .await
}

async fn fetch_or_cache<F, Fut>(
    state: AppState,
    key: String,
    sf_prefix: &'static str,
    fetcher: F,
) -> AppResult<Json<serde_json::Value>>
where
    F: FnOnce(reqwest::Client, String) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = tokimo_core::CoreResult<serde_json::Value>> + Send,
{
    if let Some(row) = NominatimGeocode::find_by_id(key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(Json(row.raw_json));
    }

    state.rate_limiter.acquire("nominatim").await?;

    let cache_key_sf = format!("{sf_prefix}:{key}");
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();
    let user_agent = state
        .config
        .nominatim_user_agent
        .clone()
        .unwrap_or_else(|| "tokimo-server/0.1 (https://github.com/tokimo-lab/tokimo-server)".into());

    let raw_json = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = NominatimGeocode::find_by_id(key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = fetcher(http, user_agent).await?;

            let am = nominatim_geocode::ActiveModel {
                cache_key: Set(key_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            NominatimGeocode::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json))
}

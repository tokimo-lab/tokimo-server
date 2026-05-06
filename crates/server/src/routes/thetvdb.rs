use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use tokimo_providers::thetvdb;

use crate::{
    db::entities::{
        thetvdb_episodes, thetvdb_series, thetvdb_token_cache, ThetvdbEpisodes, ThetvdbSeries, ThetvdbTokenCache,
    },
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/series/:id", get(get_series))
        .route("/series/:id/episodes", get(get_series_episodes))
        .route("/episode/:id", get(get_episode))
}

const TOKEN_ROW_ID: i32 = 1;

fn require_key(state: &AppState) -> AppResult<String> {
    state
        .config
        .thetvdb_api_key
        .clone()
        .ok_or_else(|| AppError::Internal("THETVDB_API_KEY not configured".into()))
}

/// Get a valid bearer token, refreshing via /login if missing or expired.
async fn ensure_token(state: &AppState) -> AppResult<String> {
    let now = chrono::Utc::now();

    let row = ThetvdbTokenCache::find_by_id(TOKEN_ROW_ID)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(r) = row {
        // Refresh ~5 minutes before stated expiry to avoid clock skew issues.
        let expires_naive = r.expires_at.naive_utc().and_utc();
        if expires_naive - chrono::Duration::minutes(5) > now {
            return Ok(r.token);
        }
    }

    // Need to refresh — call /login. Single-flight on the token refresh path
    // so concurrent route handlers don't all hammer /login at once.
    let api_key = require_key(state)?;
    let http = state.http.clone();
    let db = state.db.clone();

    let token = state
        .single_flight
        .do_once::<String, _, _>("thetvdb:login", move || async move {
            // Re-check inside single-flight; another process may have refreshed.
            let row = ThetvdbTokenCache::find_by_id(TOKEN_ROW_ID)
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;
            let now = chrono::Utc::now();
            if let Some(r) = row {
                if r.expires_at.naive_utc().and_utc() - chrono::Duration::minutes(5) > now {
                    return Ok(r.token);
                }
            }

            let token = thetvdb::login(&http, &api_key, None).await?;
            let expires_at = now + chrono::Duration::seconds(thetvdb::TOKEN_TTL_SECONDS);

            let am = thetvdb_token_cache::ActiveModel {
                id: Set(TOKEN_ROW_ID),
                token: Set(token.clone()),
                expires_at: Set(expires_at.into()),
            };
            ThetvdbTokenCache::insert(am)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(thetvdb_token_cache::Column::Id)
                        .update_columns([
                            thetvdb_token_cache::Column::Token,
                            thetvdb_token_cache::Column::ExpiresAt,
                        ])
                        .to_owned(),
                )
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(token)
        })
        .await?;

    Ok(token)
}

async fn get_series(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Json<serde_json::Value>> {
    if let Some(row) = ThetvdbSeries::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(Json(row.raw_json));
    }

    state.rate_limiter.acquire("thetvdb").await?;
    let token = ensure_token(&state).await?;

    let cache_key = format!("thetvdb:series:{}", id);
    let http = state.http.clone();
    let db = state.db.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = ThetvdbSeries::find_by_id(id)
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = thetvdb::fetch_series(&http, &token, id).await?;

            let am = thetvdb_series::ActiveModel {
                id: Set(id),
                raw_json: Set(raw.clone()),
                episodes_raw_json: Set(None),
                episodes_fetched_at: Set(None),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            ThetvdbSeries::insert(am)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(thetvdb_series::Column::Id)
                        .update_columns([thetvdb_series::Column::RawJson, thetvdb_series::Column::FetchedAt])
                        .to_owned(),
                )
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json))
}

async fn get_series_episodes(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Json<serde_json::Value>> {
    if let Some(row) = ThetvdbSeries::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        if let Some(ep_json) = row.episodes_raw_json.clone() {
            if row.episodes_fetched_at.is_some() {
                return Ok(Json(ep_json));
            }
        }
    }

    state.rate_limiter.acquire("thetvdb").await?;
    let token = ensure_token(&state).await?;

    let cache_key = format!("thetvdb:series:{}:episodes", id);
    let http = state.http.clone();
    let db = state.db.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = ThetvdbSeries::find_by_id(id)
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                if row.episodes_fetched_at.is_some() {
                    if let Some(ep) = row.episodes_raw_json {
                        return Ok(ep);
                    }
                }
            }

            let raw = thetvdb::fetch_series_episodes(&http, &token, id).await?;
            let now = chrono::Utc::now();

            // Upsert the series row, ensuring required NOT NULL fields are
            // populated. If the series row doesn't exist yet, raw_json must
            // still be NOT NULL — store an empty object as a placeholder so
            // a later /series/:id call will overwrite it.
            let existing = ThetvdbSeries::find_by_id(id)
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;
            let placeholder = serde_json::json!({});
            let am = thetvdb_series::ActiveModel {
                id: Set(id),
                raw_json: Set(existing.as_ref().map(|r| r.raw_json.clone()).unwrap_or(placeholder)),
                episodes_raw_json: Set(Some(raw.clone())),
                episodes_fetched_at: Set(Some(now.into())),
                fetched_at: Set(existing.map(|r| r.fetched_at).unwrap_or_else(|| now.into())),
            };
            ThetvdbSeries::insert(am)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(thetvdb_series::Column::Id)
                        .update_columns([
                            thetvdb_series::Column::EpisodesRawJson,
                            thetvdb_series::Column::EpisodesFetchedAt,
                        ])
                        .to_owned(),
                )
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json))
}

async fn get_episode(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Json<serde_json::Value>> {
    if let Some(row) = ThetvdbEpisodes::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(Json(row.raw_json));
    }

    state.rate_limiter.acquire("thetvdb").await?;
    let token = ensure_token(&state).await?;

    let cache_key = format!("thetvdb:episode:{}", id);
    let http = state.http.clone();
    let db = state.db.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = ThetvdbEpisodes::find_by_id(id)
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = thetvdb::fetch_episode(&http, &token, id).await?;
            let series_id = raw.get("data").and_then(|d| d.get("seriesId")).and_then(|v| v.as_i64());

            let am = thetvdb_episodes::ActiveModel {
                id: Set(id),
                series_id: Set(series_id),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            ThetvdbEpisodes::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json))
}

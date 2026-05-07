use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_providers::bangumi;

use crate::{
    db::entities::{bangumi_subjects, BangumiSubjects},
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subject/:id", get(get_subject))
        .route("/search", get(get_search))
        .route("/browse", get(get_browse))
        .route("/calendar", get(get_calendar))
}

fn require_user_agent(state: &AppState) -> AppResult<String> {
    Ok(state
        .config
        .bangumi_user_agent
        .clone()
        .unwrap_or_else(|| {
            "tokimo-server/1.0 (https://github.com/tokimo-lab/tokimo-server)".to_string()
        }))
}

async fn get_subject(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Json<serde_json::Value>> {
    if let Some(row) = BangumiSubjects::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(Json(row.raw_json));
    }

    let ua = require_user_agent(&state)?;
    state.rate_limiter.acquire("bangumi").await?;

    let cache_key = format!("bangumi:subject:{}", id);
    let http = state.http.clone();
    let db = state.db.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = BangumiSubjects::find_by_id(id)
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = match bangumi::fetch_subject(&http, &ua, None, id).await? {
                Some(v) => v,
                None => return Err(tokimo_core::CoreError::NotFound),
            };

            let am = bangumi_subjects::ActiveModel {
                id: Set(id),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            BangumiSubjects::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(rename = "type")]
    pub type_: Option<u8>,
    pub limit: Option<u32>,
}

async fn get_search(State(state): State<AppState>, Query(q): Query<SearchQuery>) -> AppResult<Json<serde_json::Value>> {
    let ua = require_user_agent(&state)?;
    state.rate_limiter.acquire("bangumi").await?;

    let subject_type = q.type_.unwrap_or(2);
    let limit = q.limit.unwrap_or(20);
    let cache_key = format!("bangumi:search:{}:{}:{}", q.q, subject_type, limit);

    let http = state.http.clone();
    let q_owned = q.q.clone();

    let resp = state
        .single_flight
        .do_once(&cache_key, move || async move {
            bangumi::search(&http, &ua, None, &q_owned, subject_type, limit).await
        })
        .await?;

    Ok(Json(resp))
}

#[derive(Deserialize)]
pub struct BrowseQuery {
    #[serde(rename = "type")]
    pub type_: Option<u8>,
    pub limit: Option<u32>,
}

async fn get_browse(State(state): State<AppState>, Query(q): Query<BrowseQuery>) -> AppResult<Json<serde_json::Value>> {
    let ua = require_user_agent(&state)?;
    state.rate_limiter.acquire("bangumi").await?;

    let subject_type = q.type_.unwrap_or(2);
    let limit = q.limit.unwrap_or(20);
    let cache_key = format!("bangumi:browse:{}:{}", subject_type, limit);

    let http = state.http.clone();

    let resp = state
        .single_flight
        .do_once(&cache_key, move || async move {
            bangumi::browse(&http, &ua, None, subject_type, limit).await
        })
        .await?;

    Ok(Json(resp))
}

async fn get_calendar(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let ua = require_user_agent(&state)?;
    state.rate_limiter.acquire("bangumi").await?;

    let cache_key = "bangumi:calendar".to_string();
    let http = state.http.clone();

    let resp = state
        .single_flight
        .do_once(&cache_key, move || async move {
            bangumi::fetch_calendar(&http, &ua, None).await
        })
        .await?;

    Ok(Json(resp))
}

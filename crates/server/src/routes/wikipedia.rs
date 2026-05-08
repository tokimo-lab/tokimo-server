use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_providers::wikipedia;

use crate::{
    db::entities::{wikipedia_summaries, WikipediaSummaries},
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/summary", get(get_summary))
}

#[derive(Deserialize)]
pub struct SummaryQuery {
    pub title: String,
    pub lang: Option<String>,
}

async fn get_summary(State(state): State<AppState>, Query(q): Query<SummaryQuery>) -> AppResult<Response> {
    let lang = q.lang.clone().unwrap_or_else(|| "en".to_string());
    let title = q.title.clone();
    let key = wikipedia::cache_key(&lang, &title);

    if let Some(row) = WikipediaSummaries::find_by_id(key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(cache_hit(Json(row.raw_json)));
    }

    state.rate_limiter.acquire("wikipedia").await?;

    let cache_key_sf = format!("wikipedia:summary:{}", key);
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();
    let lang_clone = lang.clone();
    let title_clone = title.clone();

    let raw = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = WikipediaSummaries::find_by_id(key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = wikipedia::fetch_summary(&http, &lang_clone, &title_clone).await?;

            let am = wikipedia_summaries::ActiveModel {
                cache_key: Set(key_clone.clone()),
                lang: Set(lang_clone.clone()),
                title: Set(title_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            WikipediaSummaries::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw).into_response())
}

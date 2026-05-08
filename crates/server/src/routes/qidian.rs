use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_providers::qidian;

use crate::{
    db::entities::{qidian_books, QidianBooks},
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/book/:id", get(get_book))
        .route("/search", get(get_search))
}

async fn get_book(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Response> {
    if let Some(row) = QidianBooks::find_by_id(id.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(cache_hit(Json(row.raw_json)));
    }

    state.rate_limiter.acquire("qidian").await?;

    let cache_key = format!("qidian:book:{}", id);
    let http = state.http.clone();
    let db = state.db.clone();
    let id_clone = id.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key, move || async move {
            if let Some(row) = QidianBooks::find_by_id(id_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let detail = match qidian::get_book_detail(&http, &id_clone).await? {
                Some(d) => d,
                None => return Err(tokimo_core::CoreError::NotFound),
            };
            let raw = serde_json::to_value(&detail)
                .map_err(|e| tokimo_core::CoreError::Provider(format!("qidian serialize: {e}")))?;

            let am = qidian_books::ActiveModel {
                qidian_id: Set(id_clone.clone()),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            QidianBooks::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json).into_response())
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

async fn get_search(State(state): State<AppState>, Query(q): Query<SearchQuery>) -> AppResult<Json<serde_json::Value>> {
    state.rate_limiter.acquire("qidian").await?;

    let cache_key = format!("qidian:search:{}", q.q);
    let http = state.http.clone();
    let q_owned = q.q.clone();

    let items = state
        .single_flight
        .do_once(&cache_key, move || async move {
            let items = qidian::search_books(&http, &q_owned).await?;
            serde_json::to_value(&items).map_err(|e| tokimo_core::CoreError::Provider(format!("qidian serialize: {e}")))
        })
        .await?;

    Ok(Json(items))
}

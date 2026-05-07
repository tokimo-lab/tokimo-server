use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_providers::github_releases;

use crate::{
    db::entities::{github_releases as github_releases_entity, GithubReleases},
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/releases/:owner/:repo/latest", get(get_latest))
        .route("/releases/:owner/:repo/list", get(get_list))
}

async fn get_latest(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
) -> AppResult<Json<serde_json::Value>> {
    let key = format!("{}/{}/latest", owner, repo);

    if let Some(row) = GithubReleases::find_by_id(key.clone())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(Json(row.raw_json));
    }

    state.rate_limiter.acquire("github").await?;

    let cache_key_sf = format!("github:releases:latest:{}", key);
    let http = state.http.clone();
    let db = state.db.clone();
    let key_clone = key.clone();
    let owner_owned = owner.clone();
    let repo_owned = repo.clone();
    let token = state.config.github_token.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = GithubReleases::find_by_id(key_clone.clone())
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = github_releases::fetch_latest_release(&http, &owner_owned, &repo_owned, token.as_deref()).await?;

            let tag = raw
                .get("tag_name")
                .and_then(|v| v.as_str())
                .unwrap_or("latest")
                .to_string();

            let am = github_releases_entity::ActiveModel {
                cache_key: Set(key_clone.clone()),
                owner: Set(owner_owned.clone()),
                repo: Set(repo_owned.clone()),
                tag: Set(tag),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            GithubReleases::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json))
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub per_page: Option<u32>,
    pub page: Option<u32>,
}

/// List endpoint is not persisted (high cardinality across pagination); we
/// only apply the rate limiter + single-flight to coalesce identical
/// concurrent queries.
async fn get_list(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    state.rate_limiter.acquire("github").await?;

    let cache_key_sf = format!(
        "github:releases:list:{}/{}:{}:{}",
        owner,
        repo,
        q.per_page.unwrap_or(30),
        q.page.unwrap_or(1),
    );
    let http = state.http.clone();
    let owner_owned = owner.clone();
    let repo_owned = repo.clone();
    let per_page = q.per_page;
    let page = q.page;
    let token = state.config.github_token.clone();

    let raw = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            github_releases::list_releases(&http, &owner_owned, &repo_owned, per_page, page, token.as_deref()).await
        })
        .await?;

    Ok(Json(raw))
}

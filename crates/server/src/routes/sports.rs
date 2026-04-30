use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::time::Duration;
use tokimo_providers::baidu_sports::{fetch_schedule, SportSchedule};
use tokio::time::interval;

use crate::{AppResult, AppState};

pub fn routes() -> Router<AppState> {
    Router::new().route("/schedule", get(get_schedule))
}

#[derive(Deserialize)]
struct ScheduleQuery {
    #[serde(rename = "type")]
    match_type: String,
    date: String,
}

async fn get_schedule(
    State(state): State<AppState>,
    Query(query): Query<ScheduleQuery>,
) -> AppResult<Json<SportSchedule>> {
    let cache_key = format!("sports:{}:{}", query.match_type, query.date);

    // Check cache (60s TTL)
    if let Some(cached) = state.cache.get("sports", &cache_key).await? {
        if let Ok(schedule) = serde_json::from_slice::<SportSchedule>(&cached) {
            return Ok(Json(schedule));
        }
    }

    state.rate_limiter.acquire("baidu_sports").await?;

    let http = state.http.clone();
    let match_type = query.match_type.clone();
    let date = query.date.clone();
    let cache = state.cache.clone();
    let cache_key_for_closure = cache_key.clone();

    let schedule: SportSchedule = state
        .single_flight
        .do_once(&cache_key, move || async move {
            // Race contract: must re-check provider table inside single-flight to
            // handle cross-process losers. For sports the "provider table" is the
            // shared PG cache — the first process writes it before releasing the
            // advisory lock, so losers find it here and short-circuit.
            if let Some(cached) = cache.get("sports", &cache_key_for_closure).await? {
                if let Ok(schedule) = serde_json::from_slice::<SportSchedule>(&cached) {
                    return Ok(schedule);
                }
            }

            let span =
                tracing::info_span!("upstream", provider = "baidu_sports", match_type = %match_type, date = %date);
            let _enter = span.enter();
            let schedule = fetch_schedule(&http, &match_type, &date).await?;

            if let Ok(serialized) = serde_json::to_vec(&schedule) {
                let _ = cache
                    .set(
                        "sports",
                        &cache_key_for_closure,
                        serialized.into(),
                        Duration::from_secs(60),
                    )
                    .await;
            }

            Ok(schedule)
        })
        .await?;

    Ok(Json(schedule))
}

pub async fn prewarm_task(state: AppState) {
    let mut ticker = interval(Duration::from_secs(300)); // 5 minutes

    loop {
        ticker.tick().await;

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let tomorrow = (chrono::Local::now() + chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();

        for date in &[today, tomorrow] {
            let cache_key = format!("sports:hot:{}", date);

            if state.rate_limiter.try_acquire("baidu_sports").await.unwrap_or(false) {
                let http = state.http.clone();
                let date_clone = date.clone();

                if let Ok(schedule) = fetch_schedule(&http, "hot", &date_clone).await {
                    let _ = state
                        .cache
                        .set(
                            "sports",
                            &cache_key,
                            serde_json::to_vec(&schedule).unwrap_or_default().into(),
                            Duration::from_secs(60),
                        )
                        .await;
                    tracing::debug!("Prewarmed sports schedule for {}", date);
                }
            }
        }
    }
}

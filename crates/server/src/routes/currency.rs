use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use serde::Deserialize;
use tokimo_core::CoreError;
use tokimo_providers::currency;

use crate::{
    db::entities::{currency_rates, CurrencyRates},
    metrics::cache_hit,
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/rates", get(get_rates))
}

#[derive(Deserialize)]
pub struct RatesQuery {
    pub base: Option<String>,
    pub targets: String,
    pub days: Option<u16>,
}

async fn get_rates(State(state): State<AppState>, Query(q): Query<RatesQuery>) -> AppResult<Response> {
    let base = normalize_code(q.base.as_deref().unwrap_or("USD"))
        .ok_or_else(|| AppError::BadRequest("Invalid base currency".to_string()))?;
    let days = q.days.unwrap_or(7);
    if !(1..=30).contains(&days) {
        return Err(AppError::BadRequest("days must be between 1 and 30".to_string()));
    }

    let targets = normalize_targets(&q.targets, &base)?;
    let targets_csv = targets.join(",");
    let days_i32 = i32::from(days);
    let ttl_seconds = state.provider_ttl("currency").await;

    let existing = CurrencyRates::find_by_id((base.clone(), targets_csv.clone(), days_i32))
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if let Some(row) = existing.as_ref().filter(|row| is_fresh(row.fetched_at, ttl_seconds)) {
        return Ok(cache_hit(Json(row.raw_json.clone())));
    }

    state.rate_limiter.acquire("currency").await?;

    let sf_bucket = chrono::Utc::now().timestamp() / ttl_seconds.max(1);
    let cache_key_sf = format!("currency:{base}:{targets_csv}:{days}:{sf_bucket}");
    let http = state.http.clone();
    let db = state.db.clone();
    let base_clone = base.clone();
    let targets_clone = targets.clone();
    let targets_csv_clone = targets_csv.clone();
    let existing_for_fallback = existing.clone();

    let raw = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = CurrencyRates::find_by_id((base_clone.clone(), targets_csv_clone.clone(), days_i32))
                .one(&db)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?
            {
                if is_fresh(row.fetched_at, ttl_seconds) {
                    return Ok(row.raw_json);
                }
            }

            match currency::fetch_rates(&http, &base_clone, &targets_clone, days).await {
                Ok(raw) => {
                    let am = currency_rates::ActiveModel {
                        base: Set(base_clone.clone()),
                        targets_csv: Set(targets_csv_clone.clone()),
                        days: Set(days_i32),
                        raw_json: Set(raw.clone()),
                        fetched_at: Set(chrono::Utc::now().into()),
                    };
                    CurrencyRates::insert(am)
                        .on_conflict(
                            sea_orm::sea_query::OnConflict::columns([
                                currency_rates::Column::Base,
                                currency_rates::Column::TargetsCsv,
                                currency_rates::Column::Days,
                            ])
                            .update_columns([currency_rates::Column::RawJson, currency_rates::Column::FetchedAt])
                            .to_owned(),
                        )
                        .exec(&db)
                        .await
                        .map_err(|e| CoreError::Database(e.to_string()))?;
                    Ok(raw)
                }
                Err(err) if should_fallback(&err) => {
                    if let Some(row) = existing_for_fallback {
                        tracing::warn!(
                            provider = "currency",
                            key = %format!("{}:{}:{}", base_clone, targets_csv_clone, days),
                            error = %err,
                            "returning stale currency cache after upstream failure"
                        );
                        Ok(row.raw_json)
                    } else {
                        Err(err)
                    }
                }
                Err(err) => Err(err),
            }
        })
        .await
        .or_else(|err| {
            if should_fallback(&err) {
                if let Some(row) = existing {
                    tracing::warn!(
                        provider = "currency",
                        key = %format!("{}:{}:{}", base, targets_csv, days),
                        error = %err,
                        "returning stale currency cache after upstream failure"
                    );
                    return Ok(row.raw_json);
                }
            }
            Err(err)
        })?;

    Ok(Json(raw).into_response())
}

fn normalize_targets(targets: &str, base: &str) -> AppResult<Vec<String>> {
    let mut values = Vec::new();
    for target in targets.split(',') {
        let code = normalize_code(target)
            .ok_or_else(|| AppError::BadRequest(format!("Invalid target currency: {}", target.trim())))?;
        if code != base {
            values.push(code);
        }
    }
    values.sort();
    values.dedup();

    if values.is_empty() {
        return Err(AppError::BadRequest(
            "targets must include at least one currency different from base".to_string(),
        ));
    }
    Ok(values)
}

fn normalize_code(value: &str) -> Option<String> {
    let code = value.trim().to_ascii_uppercase();
    if code.len() == 3 && code.bytes().all(|b| b.is_ascii_alphabetic()) {
        Some(code)
    } else {
        None
    }
}

fn is_fresh(fetched_at: chrono::DateTime<chrono::FixedOffset>, ttl_seconds: i64) -> bool {
    chrono::Utc::now().signed_duration_since(fetched_at) < chrono::Duration::seconds(ttl_seconds)
}

fn should_fallback(err: &CoreError) -> bool {
    match err {
        CoreError::Upstream(_) => true,
        CoreError::Provider(message) => contains_5xx_status(message),
        _ => false,
    }
}

fn contains_5xx_status(message: &str) -> bool {
    [
        " 500", " 501", " 502", " 503", " 504", " 505", " 506", " 507", " 508", " 510", " 511",
    ]
    .iter()
    .any(|status| message.contains(status))
}

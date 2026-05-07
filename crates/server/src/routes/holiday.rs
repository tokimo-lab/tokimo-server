use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use sea_orm::{entity::*, EntityTrait};
use tokimo_providers::holiday;

use crate::{
    db::entities::{holiday_years, HolidayYears},
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/:country/:year", get(get_holidays))
}

async fn get_holidays(
    State(state): State<AppState>,
    Path((country, year)): Path<(String, u16)>,
) -> AppResult<Json<serde_json::Value>> {
    let country_norm = country.to_ascii_uppercase();
    let year_i32 = year as i32;

    if let Some(row) = HolidayYears::find_by_id((country_norm.clone(), year_i32))
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        return Ok(Json(row.raw_json));
    }

    state.rate_limiter.acquire("holiday").await?;

    let cache_key_sf = format!("holiday:{country_norm}:{year}");
    let http = state.http.clone();
    let db = state.db.clone();
    let country_clone = country_norm.clone();

    let raw_json = state
        .single_flight
        .do_once(&cache_key_sf, move || async move {
            if let Some(row) = HolidayYears::find_by_id((country_clone.clone(), year_i32))
                .one(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?
            {
                return Ok(row.raw_json);
            }

            let raw = holiday::fetch_holidays(&http, &country_clone, year).await?;
            let source = holiday::source_for(&country_clone).to_string();

            let am = holiday_years::ActiveModel {
                country: Set(country_clone.clone()),
                year: Set(year_i32),
                source: Set(source),
                raw_json: Set(raw.clone()),
                fetched_at: Set(chrono::Utc::now().into()),
            };
            HolidayYears::insert(am)
                .exec(&db)
                .await
                .map_err(|e| tokimo_core::CoreError::Database(e.to_string()))?;

            Ok(raw)
        })
        .await?;

    Ok(Json(raw_json))
}

//! timor.tech holiday API — copied + adapted from
//! tokimo/packages/rust-client-api/src/timor_holiday.rs
//!
//! Differences from the upstream client:
//! - Returns raw JSON (passed through), not typed DTOs.
//! - Uses timor's bulk `/year/<year>` endpoint to enumerate public holidays
//!   for an entire year in a single round-trip.

use serde_json::Value;
use tokimo_core::{CoreError, CoreResult};

const BASE_URL: &str = "https://timor.tech/api/holiday";

/// Fetch all holidays for `year` via timor's `/year/<year>` bulk endpoint.
///
/// timor returns `{ code: 0, holiday: { "MM-DD": { ... } } }` — we forward
/// the `holiday` field as-is and let callers handle the date-keyed map.
pub async fn fetch_year_holidays(http: &reqwest::Client, year: u16) -> CoreResult<Value> {
    let url = format!("{BASE_URL}/year/{year}");
    let resp = http.get(&url).send().await.map_err(CoreError::Upstream)?;
    if !resp.status().is_success() {
        return Err(CoreError::Provider(format!(
            "timor /year returned status {}",
            resp.status()
        )));
    }
    let body: Value = resp.json().await.map_err(CoreError::Upstream)?;
    let code = body.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
    if code != 0 {
        return Err(CoreError::Provider(format!("timor API returned code {code}")));
    }
    Ok(body.get("holiday").cloned().unwrap_or(Value::Null))
}

/// Fetch the next upcoming holiday (raw JSON).
pub async fn next_holiday(http: &reqwest::Client, date: Option<&str>) -> CoreResult<Value> {
    let url = match date {
        Some(d) => format!("{BASE_URL}/next/{d}"),
        None => format!("{BASE_URL}/next"),
    };
    fetch_with_code(http, &url).await
}

/// Fetch the day-info entry for `date` (raw JSON).
pub async fn day_info(http: &reqwest::Client, date: Option<&str>) -> CoreResult<Value> {
    let url = match date {
        Some(d) => format!("{BASE_URL}/info/{d}"),
        None => format!("{BASE_URL}/info"),
    };
    fetch_with_code(http, &url).await
}

async fn fetch_with_code(http: &reqwest::Client, url: &str) -> CoreResult<Value> {
    let resp = http.get(url).send().await.map_err(CoreError::Upstream)?;
    if !resp.status().is_success() {
        return Err(CoreError::Provider(format!("timor returned status {}", resp.status())));
    }
    let body: Value = resp.json().await.map_err(CoreError::Upstream)?;
    let code = body.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
    if code != 0 {
        return Err(CoreError::Provider(format!("timor API returned code {code}")));
    }
    Ok(body)
}

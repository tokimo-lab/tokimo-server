use std::collections::BTreeMap;

use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use tokimo_core::{CoreError, CoreResult};

const FRANKFURTER_BASE: &str = "https://api.frankfurter.dev/v1";

#[derive(Deserialize)]
struct FrankfurterResponse {
    rates: BTreeMap<String, BTreeMap<String, f64>>,
}

pub async fn fetch_rates(http: &reqwest::Client, base: &str, targets: &[String], days: u16) -> CoreResult<Value> {
    let end = Utc::now().date_naive();
    let start = end - Duration::days(i64::from(days) + 4);
    let symbols = targets.join(",");
    let url = format!("{FRANKFURTER_BASE}/{start}..{end}");

    let resp = http
        .get(url)
        .query(&[("base", base), ("symbols", symbols.as_str())])
        .send()
        .await
        .map_err(CoreError::Upstream)?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(CoreError::Provider(format!(
            "Frankfurter returned status {status}: {body}"
        )));
    }

    let payload = resp.json::<FrankfurterResponse>().await.map_err(CoreError::Upstream)?;
    build_response(base, targets, days, payload.rates)
}

fn build_response(
    base: &str,
    targets: &[String],
    days: u16,
    rates_by_date: BTreeMap<String, BTreeMap<String, f64>>,
) -> CoreResult<Value> {
    let window_start = rates_by_date.len().saturating_sub(days as usize);
    let window = rates_by_date.iter().skip(window_start).collect::<Vec<_>>();
    let mut rates = Map::new();

    for target in targets {
        let history = window
            .iter()
            .filter_map(|(date, rates_for_date)| {
                rates_for_date
                    .get(target)
                    .map(|rate| json!({ "date": date, "rate": rate }))
            })
            .collect::<Vec<_>>();
        if history.is_empty() {
            continue;
        }

        let rate = history
            .last()
            .and_then(|entry| entry.get("rate"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        let prev_rate = history
            .iter()
            .rev()
            .nth(1)
            .and_then(|entry| entry.get("rate"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        let change_pct = if prev_rate > 0.0 {
            (rate - prev_rate) / prev_rate * 100.0
        } else {
            0.0
        };

        rates.insert(
            target.clone(),
            json!({
                "rate": rate,
                "prev_rate": prev_rate,
                "change_pct": change_pct,
                "history": history,
            }),
        );
    }

    if rates.is_empty() {
        return Err(CoreError::Provider(
            "Frankfurter returned no matched currency series".to_string(),
        ));
    }

    Ok(json!({
        "base": base,
        "fetched_at": Utc::now().to_rfc3339(),
        "rates": rates,
    }))
}

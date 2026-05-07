//! Public-holiday adapters: Timor (China-specific) + Nager.Date (everything else).
//!
//! Re-exposed as a single `fetch_holidays(country, year)` aggregator that
//! prefers Timor for `CN` and falls back to Nager for other countries.

pub mod nager;
pub mod timor;

use serde_json::{json, Value};
use tokimo_core::CoreResult;

/// Fetch the public-holiday list for `(country, year)`.
///
/// - For `CN` we call timor.tech and re-shape the response as a single JSON
///   object `{ "source": "timor", "country": "CN", "year": Y, "holidays": ... }`.
/// - For every other ISO-3166 alpha-2 country code we call Nager.Date and
///   wrap the result as `{ "source": "nager", "country": X, "year": Y, "holidays": [...] }`.
pub async fn fetch_holidays(http: &reqwest::Client, country: &str, year: u16) -> CoreResult<Value> {
    let upper = country.to_ascii_uppercase();
    if upper == "CN" {
        let raw = timor::fetch_year_holidays(http, year).await?;
        Ok(json!({
            "source": "timor",
            "country": "CN",
            "year": year,
            "holidays": raw,
        }))
    } else {
        let raw = nager::fetch_public_holidays(http, year, &upper).await?;
        Ok(json!({
            "source": "nager",
            "country": upper,
            "year": year,
            "holidays": raw,
        }))
    }
}

/// Source label persisted alongside cached rows.
pub fn source_for(country: &str) -> &'static str {
    if country.eq_ignore_ascii_case("CN") {
        "timor"
    } else {
        "nager"
    }
}

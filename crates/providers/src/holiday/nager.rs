//! Nager.Date — free public-holiday API.
//!
//! Copied + adapted from
//! tokimo/packages/rust-client-api/src/metadata_providers/nager_date.rs.
//! Differences from the upstream client:
//! - Returns raw JSON; the in-memory `RequestCache` is dropped because
//!   tokimo-server caches at the table layer.
//! - Only the endpoint the tokimo-server holiday route actually needs is
//!   ported: public holidays for a given year + country.

use serde_json::Value;
use tokimo_core::{CoreError, CoreResult};

const BASE_URL: &str = "https://date.nager.at/api/v3";

/// `GET /PublicHolidays/{year}/{country}` → raw JSON array.
pub async fn fetch_public_holidays(http: &reqwest::Client, year: u16, country_code: &str) -> CoreResult<Value> {
    let url = format!("{BASE_URL}/PublicHolidays/{year}/{country_code}");
    let resp = http.get(&url).send().await.map_err(CoreError::Upstream)?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(CoreError::Provider(format!(
            "Nager.Date returned status {status}: {body}"
        )));
    }
    resp.json::<Value>().await.map_err(CoreError::Upstream)
}

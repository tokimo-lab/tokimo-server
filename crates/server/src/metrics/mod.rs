use std::collections::{BTreeMap, VecDeque};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::HeaderValue;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Response header used to signal a DB-cache hit to the
/// `record_metrics` middleware. Routes that short-circuit on a local cache
/// hit should wrap their response with [`cache_hit`] so the middleware can
/// tag the recorded `MetricSample` with `cache_hit: true`.
pub const CACHE_HIT_HEADER: &str = "x-cache";

/// Wrap `body` with the `x-cache: HIT` response header. The
/// `record_metrics` middleware reads this header (case-insensitively) when
/// recording the per-request `MetricSample`, so this is the only signal a
/// route needs to emit on a cache-hit early return.
pub fn cache_hit<R: IntoResponse>(body: R) -> Response {
    let mut resp = body.into_response();
    resp.headers_mut()
        .insert(CACHE_HIT_HEADER, HeaderValue::from_static("HIT"));
    resp
}

const RING_CAPACITY: usize = 100_000;

#[derive(Serialize, Clone)]
pub struct MetricSample {
    pub ts_unix: i64,
    pub provider: String,
    pub status: u16,
    pub duration_ms: u32,
    pub cache_hit: bool,
}

pub struct MetricsStore {
    inner: RwLock<VecDeque<MetricSample>>,
}

#[derive(Serialize)]
pub struct TimeseriesBucket {
    pub ts: i64,
    pub calls: u64,
    pub errors: u64,
    pub hits: u64,
    pub misses: u64,
    pub p50_ms: u32,
    pub p95_ms: u32,
}

#[derive(Serialize)]
pub struct HeatmapCell {
    pub provider: String,
    pub calls: u64,
}

#[derive(Serialize)]
pub struct HeatmapBucket {
    pub ts: i64,
    pub values: Vec<HeatmapCell>,
}

#[derive(Serialize)]
pub struct HeatmapResponse {
    pub providers: Vec<String>,
    pub buckets: Vec<HeatmapBucket>,
}

#[derive(Serialize)]
pub struct StatusCodeBucket {
    pub ts: i64,
    pub ok_2xx: u64,
    pub client_4xx: u64,
    pub server_5xx: u64,
}

#[derive(Serialize)]
pub struct ProviderStats {
    pub provider: String,
    pub calls: u64,
    pub errors: u64,
    pub p50_ms: u32,
    pub p95_ms: u32,
    pub hit_ratio: f64,
}

#[derive(Serialize)]
pub struct ErrorSample {
    pub ts: i64,
    pub provider: String,
    pub status: u16,
    pub duration_ms: u32,
}

#[derive(Serialize)]
pub struct OverviewStats {
    pub calls_24h: u64,
    pub errors_24h: u64,
    pub hit_ratio_24h: f64,
}

impl MetricsStore {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(VecDeque::with_capacity(RING_CAPACITY)),
        }
    }

    pub fn record(&self, sample: MetricSample) {
        let mut samples = self.inner.write().unwrap_or_else(|err| err.into_inner());
        if samples.len() >= RING_CAPACITY {
            samples.pop_front();
        }
        samples.push_back(sample);
    }

    /// Remove samples whose `ts_unix` falls inside the inclusive
    /// `[since_unix, until_unix]` window. `None` means unbounded on that side
    /// (so `clear_in_range(None, None)` empties the entire ring). Returns
    /// the number of samples removed. All work happens in a single critical
    /// section to keep readers consistent.
    pub fn clear_in_range(&self, since_unix: Option<i64>, until_unix: Option<i64>) -> usize {
        let mut samples = self.inner.write().unwrap_or_else(|err| err.into_inner());
        let before = samples.len();
        match (since_unix, until_unix) {
            (None, None) => samples.clear(),
            (since, until) => {
                samples.retain(|sample| {
                    let ts = sample.ts_unix;
                    let in_window = since.is_none_or(|s| ts >= s) && until.is_none_or(|u| ts <= u);
                    !in_window
                });
            }
        }
        before - samples.len()
    }

    pub fn overview_stats_24h(&self) -> OverviewStats {
        let start = now_unix() - 24 * 60 * 60;
        let samples = self.inner.read().unwrap_or_else(|err| err.into_inner());
        let mut calls = 0;
        let mut errors = 0;
        let mut hits = 0;

        for sample in samples.iter().filter(|sample| sample.ts_unix >= start) {
            calls += 1;
            if sample.status >= 400 {
                errors += 1;
            }
            if sample.cache_hit {
                hits += 1;
            }
        }

        OverviewStats {
            calls_24h: calls,
            errors_24h: errors,
            hit_ratio_24h: ratio(hits, calls),
        }
    }

    pub fn query_timeseries(&self, range_secs: i64, bucket_secs: i64) -> Vec<TimeseriesBucket> {
        let range_secs = range_secs.max(1);
        let bucket_secs = bucket_secs.max(1);
        let now = now_unix();
        let start = now - range_secs;
        let bucket_count = ((range_secs + bucket_secs - 1) / bucket_secs).max(1) as usize;

        let mut buckets = (0..bucket_count)
            .map(|idx| TimeseriesBucket {
                ts: start + idx as i64 * bucket_secs,
                calls: 0,
                errors: 0,
                hits: 0,
                misses: 0,
                p50_ms: 0,
                p95_ms: 0,
            })
            .collect::<Vec<_>>();
        let mut durations: Vec<Vec<u32>> = (0..bucket_count).map(|_| Vec::new()).collect();

        let samples = self.inner.read().unwrap_or_else(|err| err.into_inner());
        for sample in samples.iter().filter(|sample| sample.ts_unix >= start) {
            let raw_index = (sample.ts_unix - start) / bucket_secs;
            let index = raw_index.clamp(0, bucket_count as i64 - 1) as usize;
            let bucket = &mut buckets[index];
            bucket.calls += 1;
            if sample.status >= 400 {
                bucket.errors += 1;
            }
            if sample.cache_hit {
                bucket.hits += 1;
            } else {
                bucket.misses += 1;
            }
            durations[index].push(sample.duration_ms);
        }

        for (bucket, durs) in buckets.iter_mut().zip(durations.iter_mut()) {
            durs.sort_unstable();
            bucket.p50_ms = percentile(durs, 0.50);
            bucket.p95_ms = percentile(durs, 0.95);
        }

        buckets
    }

    pub fn query_heatmap(&self, range_secs: i64, bucket_secs: i64) -> HeatmapResponse {
        let range_secs = range_secs.max(1);
        let bucket_secs = bucket_secs.max(1);
        let now = now_unix();
        let start = now - range_secs;
        let bucket_count = ((range_secs + bucket_secs - 1) / bucket_secs).max(1) as usize;

        let samples = self.inner.read().unwrap_or_else(|err| err.into_inner());

        let mut totals: BTreeMap<String, u64> = BTreeMap::new();
        for sample in samples.iter().filter(|sample| sample.ts_unix >= start) {
            *totals.entry(sample.provider.clone()).or_insert(0) += 1;
        }
        let mut ranked: Vec<(String, u64)> = totals.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        ranked.truncate(10);
        let providers: Vec<String> = ranked.into_iter().map(|(p, _)| p).collect();
        let provider_set: std::collections::HashSet<&str> = providers.iter().map(|p| p.as_str()).collect();

        let mut bucket_counts: Vec<BTreeMap<String, u64>> = (0..bucket_count).map(|_| BTreeMap::new()).collect();
        for sample in samples.iter().filter(|sample| sample.ts_unix >= start) {
            if !provider_set.contains(sample.provider.as_str()) {
                continue;
            }
            let raw_index = (sample.ts_unix - start) / bucket_secs;
            let index = raw_index.clamp(0, bucket_count as i64 - 1) as usize;
            *bucket_counts[index].entry(sample.provider.clone()).or_insert(0) += 1;
        }

        let buckets = bucket_counts
            .into_iter()
            .enumerate()
            .map(|(idx, counts)| {
                let mut values: Vec<HeatmapCell> = counts
                    .into_iter()
                    .map(|(provider, calls)| HeatmapCell { provider, calls })
                    .collect();
                values.sort_by(|a, b| b.calls.cmp(&a.calls).then_with(|| a.provider.cmp(&b.provider)));
                HeatmapBucket {
                    ts: start + idx as i64 * bucket_secs,
                    values,
                }
            })
            .collect();

        HeatmapResponse { providers, buckets }
    }

    pub fn query_status_codes(&self, range_secs: i64, bucket_secs: i64) -> Vec<StatusCodeBucket> {
        let range_secs = range_secs.max(1);
        let bucket_secs = bucket_secs.max(1);
        let now = now_unix();
        let start = now - range_secs;
        let bucket_count = ((range_secs + bucket_secs - 1) / bucket_secs).max(1) as usize;

        let mut buckets = (0..bucket_count)
            .map(|idx| StatusCodeBucket {
                ts: start + idx as i64 * bucket_secs,
                ok_2xx: 0,
                client_4xx: 0,
                server_5xx: 0,
            })
            .collect::<Vec<_>>();

        let samples = self.inner.read().unwrap_or_else(|err| err.into_inner());
        for sample in samples.iter().filter(|sample| sample.ts_unix >= start) {
            let raw_index = (sample.ts_unix - start) / bucket_secs;
            let index = raw_index.clamp(0, bucket_count as i64 - 1) as usize;
            let bucket = &mut buckets[index];
            match sample.status {
                200..=299 => bucket.ok_2xx += 1,
                400..=499 => bucket.client_4xx += 1,
                s if s >= 500 => bucket.server_5xx += 1,
                _ => {}
            }
        }

        buckets
    }

    pub fn query_by_provider(&self, range_secs: i64) -> Vec<ProviderStats> {
        let start = now_unix() - range_secs.max(1);
        let samples = self.inner.read().unwrap_or_else(|err| err.into_inner());
        let mut grouped: BTreeMap<String, ProviderAccumulator> = BTreeMap::new();

        for sample in samples.iter().filter(|sample| sample.ts_unix >= start) {
            let accumulator = grouped.entry(sample.provider.clone()).or_default();
            accumulator.calls += 1;
            if sample.status >= 400 {
                accumulator.errors += 1;
            }
            if sample.cache_hit {
                accumulator.hits += 1;
            }
            accumulator.durations.push(sample.duration_ms);
        }

        grouped
            .into_iter()
            .map(|(provider, mut accumulator)| {
                accumulator.durations.sort_unstable();
                ProviderStats {
                    provider,
                    calls: accumulator.calls,
                    errors: accumulator.errors,
                    p50_ms: percentile(&accumulator.durations, 0.50),
                    p95_ms: percentile(&accumulator.durations, 0.95),
                    hit_ratio: ratio(accumulator.hits, accumulator.calls),
                }
            })
            .collect()
    }

    pub fn query_recent_errors(&self, limit: usize) -> Vec<ErrorSample> {
        let samples = self.inner.read().unwrap_or_else(|err| err.into_inner());
        samples
            .iter()
            .rev()
            .filter(|sample| sample.status >= 400)
            .take(limit)
            .map(|sample| ErrorSample {
                ts: sample.ts_unix,
                provider: sample.provider.clone(),
                status: sample.status,
                duration_ms: sample.duration_ms,
            })
            .collect()
    }
}

impl Default for MetricsStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
struct ProviderAccumulator {
    calls: u64,
    errors: u64,
    hits: u64,
    durations: Vec<u32>,
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn percentile(sorted: &[u32], percentile: f64) -> u32 {
    if sorted.is_empty() {
        return 0;
    }
    let index = ((sorted.len() - 1) as f64 * percentile).ceil() as usize;
    sorted[index.min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(provider: &str, status: u16, duration_ms: u32, cache_hit: bool, ts_offset: i64) -> MetricSample {
        MetricSample {
            ts_unix: now_unix() + ts_offset,
            provider: provider.to_string(),
            status,
            duration_ms,
            cache_hit,
        }
    }

    #[test]
    fn test_timeseries_includes_percentiles() {
        let store = MetricsStore::new();
        for (i, dur) in [10u32, 20, 30, 40, 50, 60, 70, 80, 90, 100].iter().enumerate() {
            store.record(sample("wikipedia", 200, *dur, false, -(i as i64)));
        }

        let buckets = store.query_timeseries(3600, 3600);
        assert_eq!(buckets.len(), 1);
        let b = &buckets[0];
        assert_eq!(b.calls, 10);
        // p50 of [10..100] step 10 sorted -> index ceil(9*0.5)=5 -> 60
        assert_eq!(b.p50_ms, 60);
        // p95 -> index ceil(9*0.95)=9 -> 100
        assert_eq!(b.p95_ms, 100);

        // empty bucket
        let empty = MetricsStore::new().query_timeseries(3600, 3600);
        assert_eq!(empty[0].p50_ms, 0);
        assert_eq!(empty[0].p95_ms, 0);
    }

    #[test]
    fn test_heatmap_top_10_and_buckets() {
        let store = MetricsStore::new();
        // 12 providers, provider_i recorded (i+1) times -> top 10 by count, leaving out the smallest 2
        for i in 0..12u32 {
            let name = format!("provider_{:02}", i);
            for _ in 0..=i {
                store.record(sample(&name, 200, 5, false, 0));
            }
        }

        let resp = store.query_heatmap(3600, 3600);
        assert_eq!(resp.providers.len(), 10);
        // top should be provider_11 (12 calls) first
        assert_eq!(resp.providers[0], "provider_11");
        // smallest two excluded
        assert!(!resp.providers.contains(&"provider_00".to_string()));
        assert!(!resp.providers.contains(&"provider_01".to_string()));
        assert_eq!(resp.buckets.len(), 1);
        // bucket values include only top 10 (no provider_00/01)
        for cell in &resp.buckets[0].values {
            assert_ne!(cell.provider, "provider_00");
            assert_ne!(cell.provider, "provider_01");
        }
        assert_eq!(resp.buckets[0].values.len(), 10);
    }

    #[test]
    fn test_clear_in_range_bounds() {
        let store = MetricsStore::new();
        // ts offsets: -100, -50, -10, 0
        for offset in [-100i64, -50, -10, 0] {
            store.record(sample("p", 200, 1, false, offset));
        }
        let now = now_unix();

        // clear since now-30 (keeps offsets -100, -50; removes -10, 0) -> 2 removed
        let removed = store.clear_in_range(Some(now - 30), None);
        assert_eq!(removed, 2);

        // clear until now-75 (keeps -50, removes -100) -> 1 removed
        let removed = store.clear_in_range(None, Some(now - 75));
        assert_eq!(removed, 1);

        // remaining: offset -50. clear all
        let removed = store.clear_in_range(None, None);
        assert_eq!(removed, 1);
        assert_eq!(store.query_recent_errors(10).len(), 0);
    }

    #[test]
    fn test_clear_in_range_window() {
        let store = MetricsStore::new();
        for offset in [-100i64, -50, -25, -10, 0] {
            store.record(sample("p", 200, 1, false, offset));
        }
        let now = now_unix();
        // clear inclusive window [-60, -20] -> matches offsets -50, -25 (2)
        let removed = store.clear_in_range(Some(now - 60), Some(now - 20));
        assert_eq!(removed, 2);
    }

    #[test]
    fn test_status_code_buckets() {
        let store = MetricsStore::new();
        for _ in 0..3 {
            store.record(sample("p", 200, 1, false, 0));
        }
        store.record(sample("p", 204, 1, false, 0));
        store.record(sample("p", 404, 1, false, 0));
        store.record(sample("p", 429, 1, false, 0));
        store.record(sample("p", 500, 1, false, 0));
        store.record(sample("p", 503, 1, false, 0));
        // ignored: 1xx / 3xx
        store.record(sample("p", 100, 1, false, 0));
        store.record(sample("p", 301, 1, false, 0));

        let buckets = store.query_status_codes(3600, 3600);
        assert_eq!(buckets.len(), 1);
        let b = &buckets[0];
        assert_eq!(b.ok_2xx, 4);
        assert_eq!(b.client_4xx, 2);
        assert_eq!(b.server_5xx, 2);
    }
}

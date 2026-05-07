use std::collections::{BTreeMap, VecDeque};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

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
            })
            .collect::<Vec<_>>();

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

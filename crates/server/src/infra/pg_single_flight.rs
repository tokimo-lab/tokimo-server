use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use tokimo_core::{CoreError, CoreResult};
use xxhash_rust::xxh3::xxh3_64;

use super::single_flight::LocalSingleFlight;

/// Two-tier single-flight: process-local DashMap dedup + PostgreSQL
/// `pg_advisory_xact_lock` cross-process gating.
///
/// The advisory lock is held inside a dedicated transaction whose only purpose
/// is to act as a cross-process mutex. `f` is user code that runs separately;
/// `f`'s own DB writes commit on their own connections so subsequent waiters
/// observe them after acquiring the lock.
///
/// Race contract for callers: inside `f` the FIRST action MUST be a re-check
/// of the provider's persistent table. Cross-process losers wake up to find
/// the work already done and should return that result instead of re-running
/// the upstream call.
pub struct PgSingleFlight {
    local: Arc<LocalSingleFlight>,
    db: Arc<DatabaseConnection>,
}

impl PgSingleFlight {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            local: Arc::new(LocalSingleFlight::new()),
            db,
        }
    }

    pub async fn do_once<T, F, Fut>(&self, key: &str, f: F) -> CoreResult<T>
    where
        T: Clone + Send + Serialize + for<'de> Deserialize<'de> + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = CoreResult<T>> + Send + 'static,
    {
        // Phase 1: local dedup — same-process callers wait on a single in-flight
        // task. We reuse `LocalSingleFlight::do_once`; its serde-roundtrip cost is
        // acceptable for the MVP. (A future optimization could expose a
        // `do_once_no_cache` variant on Local that only dedups in-flight work
        // without caching the result, since under the PG layer we don't want
        // process-local indefinite caching anyway. The route handlers already
        // re-check DB/cache before calling `do_once`, so the stale-cache window
        // is bounded by request latency.)
        let db = Arc::clone(&self.db);
        let key_owned = key.to_string();

        self.local
            .do_once(key, move || async move {
                // Phase 2: cross-process advisory lock inside a dedicated tx.
                let lock_id = key_to_lock_id(&key_owned);
                let tx = db.begin().await.map_err(map_db_err)?;

                tx.execute(Statement::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    "SELECT pg_advisory_xact_lock($1)",
                    [lock_id.into()],
                ))
                .await
                .map_err(map_db_err)?;

                // Phase 3: run user code. `f` is responsible for re-checking the
                // provider table first (cross-process losers short-circuit here).
                let result = f().await;

                // Phase 4: commit to release the advisory lock. We commit even
                // when `f` failed so the lock doesn't hold via rollback delay;
                // the empty tx has no side effects.
                tx.commit().await.map_err(map_db_err)?;

                result
            })
            .await
    }
}

/// Stable mapping from cache key string to a postgres-friendly i64 lock id.
///
/// Uses xxh3_64 (deterministic across processes/versions) and reinterprets the
/// u64 as i64 via `as` cast (PG accepts the full int8 range, sign doesn't
/// matter — the function just compares bit patterns).
pub fn key_to_lock_id(key: &str) -> i64 {
    xxh3_64(key.as_bytes()) as i64
}

fn map_db_err(e: sea_orm::DbErr) -> CoreError {
    CoreError::Database(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    #[test]
    fn key_to_lock_id_is_deterministic() {
        let a = key_to_lock_id("tmdb:movie:550");
        let b = key_to_lock_id("tmdb:movie:550");
        assert_eq!(a, b);
    }

    #[test]
    fn key_to_lock_id_differs_per_key() {
        let a = key_to_lock_id("tmdb:movie:550");
        let b = key_to_lock_id("tmdb:movie:551");
        assert_ne!(a, b);
    }

    /// Cross-process single-flight integration test.
    ///
    /// Requires PG_TEST_URL pointing at a running postgres. Skipped by default
    /// (`#[ignore]`); run via `cargo test -p tokimo-server -- --ignored`.
    ///
    /// Two `PgSingleFlight` instances simulate two processes sharing the same
    /// PG. With 10 concurrent `do_once` calls split across them, the inner
    /// closure must run exactly once.
    #[tokio::test]
    #[ignore]
    #[serial_test::serial]
    async fn pg_single_flight_dedups_across_instances() {
        let url = match std::env::var("PG_TEST_URL") {
            Ok(v) => v,
            Err(_) => {
                eprintln!("PG_TEST_URL not set, skipping");
                return;
            }
        };

        let db = Arc::new(sea_orm::Database::connect(&url).await.expect("connect"));

        // Two independent single-flight instances backed by the same pool —
        // their LOCAL maps are disjoint, so dedup must come from PG.
        let sf_a = Arc::new(PgSingleFlight::new(Arc::clone(&db)));
        let sf_b = Arc::new(PgSingleFlight::new(Arc::clone(&db)));

        let counter = Arc::new(AtomicU32::new(0));
        let key = format!("pg_sf_test:{}", uuid::Uuid::new_v4());

        let mut handles = Vec::new();
        for i in 0..10 {
            let sf = if i % 2 == 0 {
                Arc::clone(&sf_a)
            } else {
                Arc::clone(&sf_b)
            };
            let counter = Arc::clone(&counter);
            let key = key.clone();
            handles.push(tokio::spawn(async move {
                sf.do_once(&key, move || async move {
                    // First waiter to acquire the PG lock increments. Losers wake
                    // up, find counter > 0, and short-circuit (mirroring the
                    // route-handler "re-check provider table first" contract).
                    if counter.load(Ordering::SeqCst) == 0 {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        counter.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok::<u32, CoreError>(42)
                })
                .await
            }));
        }

        for h in handles {
            let r = h.await.expect("join").expect("do_once");
            assert_eq!(r, 42);
        }

        assert_eq!(counter.load(Ordering::SeqCst), 1, "f should run exactly once");
    }
}

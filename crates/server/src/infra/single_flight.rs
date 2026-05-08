use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use tokimo_core::{CoreError, CoreResult};
use tokio::sync::{Notify, RwLock};

/// Transient handoff slot from the first caller to in-flight waiters.
///
/// On success carries the serialized payload; on failure carries the error
/// rendered via `Display` (CoreError isn't Clone). The slot lives only as
/// long as `f` is in flight — once all waiters have been notified the
/// inflight entry is removed, so subsequent callers re-execute `f`.
type ResultSlot = Arc<RwLock<Option<Result<Vec<u8>, String>>>>;

#[derive(Clone)]
struct Inflight {
    notify: Arc<Notify>,
    slot: ResultSlot,
}

pub struct LocalSingleFlight {
    inflight: DashMap<String, Inflight>,
}

impl Default for LocalSingleFlight {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalSingleFlight {
    pub fn new() -> Self {
        Self {
            inflight: DashMap::new(),
        }
    }

    // TODO: Cross-process single-flight using PostgreSQL advisory locks
    // For multi-instance deployments, replace with:
    // SELECT pg_advisory_xact_lock(hashtext($key));
    // This ensures only one instance across all processes executes the function.
    #[allow(dead_code)]
    async fn cross_process_lock(&self, _key: &str) -> CoreResult<()> {
        // Stub for future implementation
        Ok(())
    }

    /// Deduplicate concurrent calls for the same key.
    ///
    /// Only in-flight work is shared: the first caller runs `f`, hands the
    /// outcome to any waiting callers via a transient slot, and then clears
    /// the entry. Results are NOT cached across invocations — the real cache
    /// layer (DB, etc.) is the caller's responsibility, which means a
    /// transient upstream failure won't poison subsequent requests.
    pub async fn do_once<T, F, Fut>(&self, key: &str, f: F) -> CoreResult<T>
    where
        T: Clone + Send + Serialize + for<'de> Deserialize<'de> + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = CoreResult<T>> + Send + 'static,
    {
        // Join an existing in-flight call, or become the leader.
        let (inflight, is_first) = {
            let entry = self.inflight.entry(key.to_string());
            match entry {
                dashmap::mapref::entry::Entry::Occupied(e) => (e.get().clone(), false),
                dashmap::mapref::entry::Entry::Vacant(e) => {
                    let inflight = Inflight {
                        notify: Arc::new(Notify::new()),
                        slot: Arc::new(RwLock::new(None)),
                    };
                    e.insert(inflight.clone());
                    (inflight, true)
                }
            }
        };

        if is_first {
            let result = f().await;

            // Publish a serialized snapshot for waiters. Errors are rendered
            // through Display so we can hand them to multiple waiters without
            // requiring CoreError: Clone.
            let snapshot: Result<Vec<u8>, String> = match &result {
                Ok(val) => serde_json::to_vec(val).map_err(|e| format!("Serialization failed: {}", e)),
                Err(e) => Err(e.to_string()),
            };

            {
                let mut guard = inflight.slot.write().await;
                *guard = Some(snapshot);
            }

            inflight.notify.notify_waiters();
            self.inflight.remove(key);

            result
        } else {
            inflight.notify.notified().await;

            let guard = inflight.slot.read().await;
            match guard.as_ref() {
                Some(Ok(bytes)) => serde_json::from_slice::<T>(bytes)
                    .map_err(|e| CoreError::Internal(format!("Deserialization failed: {}", e))),
                Some(Err(msg)) => Err(CoreError::Internal(msg.clone())),
                None => Err(CoreError::Internal("Single-flight result disappeared".into())),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_single_flight_deduplication() {
        let sf = Arc::new(LocalSingleFlight::new());
        let counter = Arc::new(AtomicU32::new(0));

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let sf = sf.clone();
                let counter = counter.clone();
                tokio::spawn(async move {
                    sf.do_once("test_key", move || async move {
                        counter.fetch_add(1, Ordering::SeqCst);
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                        Ok(42u32)
                    })
                    .await
                })
            })
            .collect();

        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 42);
        }

        // Function should have been called exactly once
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_failure_does_not_poison() {
        let sf = Arc::new(LocalSingleFlight::new());
        let counter = Arc::new(AtomicU32::new(0));

        // First call: f returns an error. The error must surface to the
        // caller verbatim (no "Cached error:" wrapping) and must NOT be
        // retained for future callers.
        let counter1 = counter.clone();
        let res1 = sf
            .do_once::<u32, _, _>("poison_key", move || async move {
                counter1.fetch_add(1, Ordering::SeqCst);
                Err(CoreError::Provider("upstream boom".into()))
            })
            .await;
        assert!(res1.is_err());
        let err_msg = format!("{}", res1.unwrap_err());
        assert!(err_msg.contains("upstream boom"), "got: {}", err_msg);
        assert!(!err_msg.contains("Cached error"), "got: {}", err_msg);

        // Second isolated call after the first has completed: f MUST run
        // again. Counter goes from 1 -> 2 and the call now succeeds.
        let counter2 = counter.clone();
        let res2 = sf
            .do_once::<u32, _, _>("poison_key", move || async move {
                counter2.fetch_add(1, Ordering::SeqCst);
                Ok(7u32)
            })
            .await;
        assert_eq!(res2.unwrap(), 7);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
}

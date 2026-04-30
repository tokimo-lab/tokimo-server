use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use tokimo_core::{CoreError, CoreResult};
use tokio::sync::{Notify, RwLock};

type ResultCell<T> = Arc<RwLock<Option<CoreResult<T>>>>;

pub struct LocalSingleFlight {
    inflight: DashMap<String, Arc<Notify>>,
    results: DashMap<String, ResultCell<Vec<u8>>>,
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
            results: DashMap::new(),
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

    pub async fn do_once<T, F, Fut>(&self, key: &str, f: F) -> CoreResult<T>
    where
        T: Clone + Send + Serialize + for<'de> Deserialize<'de> + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = CoreResult<T>> + Send + 'static,
    {
        // Check if result already exists
        if let Some(result_cell) = self.results.get(key) {
            let lock = result_cell.read().await;
            if let Some(result) = lock.as_ref() {
                return match result {
                    Ok(bytes) => serde_json::from_slice::<T>(bytes)
                        .map_err(|e| CoreError::Internal(format!("Deserialization failed: {}", e))),
                    Err(e) => Err(CoreError::Internal(format!("Cached error: {:?}", e))),
                };
            }
        }

        // Check if already in flight
        let (notify, is_first) = {
            let entry = self.inflight.entry(key.to_string());
            match entry {
                dashmap::mapref::entry::Entry::Occupied(e) => (e.get().clone(), false),
                dashmap::mapref::entry::Entry::Vacant(e) => {
                    let notify = Arc::new(Notify::new());
                    e.insert(notify.clone());
                    (notify, true)
                }
            }
        };

        if is_first {
            // We're the first, execute the function
            let result = f().await;

            // Serialize and store result
            let bytes_result = match &result {
                Ok(val) => {
                    serde_json::to_vec(val).map_err(|e| CoreError::Internal(format!("Serialization failed: {}", e)))
                }
                Err(e) => Err(CoreError::Internal(format!("Function failed: {:?}", e))),
            };

            let result_cell = Arc::new(RwLock::new(Some(bytes_result)));
            self.results.insert(key.to_string(), result_cell);

            // Notify waiters
            notify.notify_waiters();

            // Clean up inflight marker
            self.inflight.remove(key);

            result
        } else {
            // Wait for the first caller to complete
            notify.notified().await;

            // Retrieve result
            if let Some(result_cell) = self.results.get(key) {
                let lock = result_cell.read().await;
                if let Some(result) = lock.as_ref() {
                    return match result {
                        Ok(bytes) => serde_json::from_slice::<T>(bytes)
                            .map_err(|e| CoreError::Internal(format!("Deserialization failed: {}", e))),
                        Err(e) => Err(CoreError::Internal(format!("Cached error: {:?}", e))),
                    };
                }
            }

            Err(CoreError::Internal("Single-flight result disappeared".into()))
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
}

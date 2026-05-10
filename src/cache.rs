use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
struct CacheEntry {
    value: Vec<u8>,
    expires_at: Instant,
}

#[derive(Clone)]
pub struct TtlCache {
    store: Arc<RwLock<HashMap<String, CacheEntry>>>,
    in_flight: Arc<RwLock<HashSet<String>>>,
    ttl: Duration,
}

impl TtlCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            in_flight: Arc::new(RwLock::new(HashSet::new())),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let store = self.store.read().unwrap();
        store.get(key).and_then(|entry| {
            if entry.expires_at > Instant::now() {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    // it even gives back stale cache
    pub fn get_stale(&self, key: &str) -> Option<Vec<u8>> {
        let store = self.store.read().unwrap();
        store.get(key).map(|entry| entry.value.clone())
    }

    // returns true if this stream needs to update the cache
    pub fn try_lock_inflight(&self, key: &str) -> bool {
        let mut in_flight = self.in_flight.write().unwrap();
        if in_flight.contains(key) {
            false // кто-то уже обновляет
        } else {
            in_flight.insert(key.to_string());
            true
        }
    }

    pub fn unlock_inflight(&self, key: &str) {
        let mut in_flight = self.in_flight.write().unwrap();
        in_flight.remove(key);
    }

    pub fn set(&self, key: String, value: Vec<u8>) {
        let mut store = self.store.write().unwrap();
        store.insert(key, CacheEntry {
            value,
            expires_at: Instant::now() + self.ttl,
        });
    }

    pub fn invalidate(&self, key: &str) {
        let mut store = self.store.write().unwrap();
        store.remove(key);
    }

    pub fn evict_expired(&self) {
        let mut store = self.store.write().unwrap();
        let now = Instant::now();
        store.retain(|_, entry| entry.expires_at > now);
    }

    pub fn start_eviction_task(self: &Arc<Self>) {
        let cache_clone = Arc::clone(self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                cache_clone.evict_expired();
            }
        });
    }
}
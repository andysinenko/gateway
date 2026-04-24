use std::collections::HashMap;
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
    ttl: Duration,
}

impl TtlCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let store = self.store.read().unwrap();
        store.get(key).and_then(|entry| {
            if entry.expires_at > Instant::now() {
                Some(entry.value.clone())
            } else {
                None  // протух, но ещё не удалён
            }
        })
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

    // Cleaning expired records
    pub fn evict_expired(&self) {
        let mut store = self.store.write().unwrap();
        let now = Instant::now();
        store.retain(|_, entry| entry.expires_at > now);
    }

    //schedule of cache clearing
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

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::Instant;
use crate::error::CommunexError;
use super::CacheConfig;
use std::pin::Pin;
use std::future::Future;
use std::fmt;
use tokio::time::Duration;

type RefreshHandler = Box<dyn Fn(&str) -> Pin<Box<dyn Future<Output = Result<QueryResult, CommunexError>> + Send>> + Send + Sync>;

#[derive(Debug, Clone, PartialEq)]
pub struct QueryResult {
    pub data: String,
}

impl QueryResult {
    pub fn new(data: &str) -> Self {
        Self {
            data: data.to_string(),
        }
    }
}

impl Default for QueryResult {
    fn default() -> Self {
        Self {
            data: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: QueryResult,
    expires_at: Instant,
}

#[derive(Debug, Clone, Default)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub refresh_failures: u64,
    pub current_entries: usize,
}

#[derive(Clone)]
pub struct QueryMapCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    config: CacheConfig,
    metrics: Arc<RwLock<CacheMetrics>>,
    refresh_handler: Arc<RwLock<Option<RefreshHandler>>>,
}

// Manual Debug implementation that skips the refresh_handler
impl fmt::Debug for QueryMapCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QueryMapCache")
            .field("config", &self.config)
            .field("metrics", &self.metrics)
            .field("entries_count", &self.entries.try_read().map(|e| e.len()).unwrap_or(0))
            .finish()
    }
}

impl QueryMapCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
            refresh_handler: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set(&self, key: &str, value: QueryResult) {
        let mut entries = self.entries.write().await;
        let expires_at = Instant::now() + self.config.ttl;
        
        entries.insert(key.to_string(), CacheEntry { value, expires_at });
        
        if entries.len() > self.config.max_entries {
            let oldest_key = entries.iter()
                .min_by_key(|(_, entry)| entry.expires_at)
                .map(|(k, _)| k.clone());
            
            if let Some(key) = oldest_key {
                entries.remove(&key);
            }
        }
        
        let mut metrics = self.metrics.write().await;
        metrics.current_entries = entries.len();
    }

    pub async fn get(&self, key: &str) -> Option<QueryResult> {
        let entries = self.entries.read().await;
        let mut metrics = self.metrics.write().await;
        
        if let Some(entry) = entries.get(key) {
            if entry.expires_at > Instant::now() {
                metrics.hits += 1;
                return Some(entry.value.clone());
            }
        }
        
        metrics.misses += 1;
        None
    }

    pub async fn get_metrics(&self) -> CacheMetrics {
        let metrics = self.metrics.read().await;
        (*metrics).clone()
    }

    pub async fn set_refresh_handler(&self, handler: RefreshHandler) {
        let mut refresh_handler = self.refresh_handler.write().await;
        *refresh_handler = Some(handler);
    }

    pub async fn start_background_refresh(&self) {
        let cache = Arc::new(self.clone());
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(cache.config.refresh_interval).await;
                
                // Get all keys that need refresh
                let keys_to_refresh = {
                    let entries = cache.entries.read().await;
                    entries
                        .iter()
                        .filter(|(_, entry)| entry.expires_at <= Instant::now())
                        .map(|(k, _)| k.clone())
                        .collect::<Vec<String>>()
                };
                
                // Process each key
                for key in keys_to_refresh {
                    if let Some(handler) = cache.refresh_handler.read().await.as_ref() {
                        match handler(&key).await {
                            Ok(new_value) => {
                                cache.set(&key, new_value).await;
                            },
                            Err(_) => {
                                let mut metrics = cache.metrics.write().await;
                                metrics.refresh_failures += 1;
                            }
                        }
                    }
                }
            }
        });
    }

    // Add a method to force expire an entry (useful for testing)
    #[cfg(test)]
    pub(crate) async fn force_expire(&self, key: &str) {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get_mut(key) {
            entry.expires_at = Instant::now() - Duration::from_secs(1);
        }
    }
} 
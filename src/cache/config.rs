use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub ttl: Duration,
    pub refresh_interval: Duration,
    pub max_entries: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(60),
            refresh_interval: Duration::from_secs(300),
            max_entries: 1000,
        }
    }
} 
use std::time::Duration;

pub struct QueryMapConfig {
    pub refresh_interval: Duration,
    pub cache_duration: Duration,
}

impl Default for QueryMapConfig {
    fn default() -> Self {
        Self {
            refresh_interval: Duration::from_secs(300), // 5 minutes
            cache_duration: Duration::from_secs(600),   // 10 minutes
        }
    }
} 
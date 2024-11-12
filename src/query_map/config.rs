use std::time::Duration;
use crate::error::CommunexError;

#[derive(Debug, Clone)]
pub struct QueryMapConfig {
    /// Interval between cache refreshes (minimum 1 second)
    pub refresh_interval: Duration,
    /// How long to keep cached data (must be longer than refresh_interval)
    pub cache_duration: Duration,
}

impl QueryMapConfig {
    pub fn validate(&self) -> Result<(), CommunexError> {
        if self.refresh_interval < Duration::from_secs(1) {
            return Err(CommunexError::ConfigError(
                "Refresh interval must be at least 1 second".to_string()
            ));
        }

        if self.cache_duration <= self.refresh_interval {
            return Err(CommunexError::ConfigError(
                "Cache duration must be longer than refresh interval".to_string()
            ));
        }

        Ok(())
    }
}

impl Default for QueryMapConfig {
    fn default() -> Self {
        Self {
            refresh_interval: Duration::from_secs(300), // 5 minutes
            cache_duration: Duration::from_secs(600),   // 10 minutes
        }
    }
} 
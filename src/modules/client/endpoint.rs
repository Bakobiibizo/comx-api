use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Access control level for module endpoints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccessLevel {
    /// Public endpoints can be called by anyone
    Public,
    /// Protected endpoints require authentication
    Protected,
    /// Private endpoints require both authentication and authorization
    Private,
}

/// Rate limit configuration for an endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Maximum number of requests allowed in the window
    pub max_requests: u32,
    /// Time window in seconds
    pub window_secs: u32,
}

/// Configuration for a module endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    /// Name of the endpoint
    pub name: String,
    /// Path relative to module base URL
    pub path: String,
    /// Required access level
    pub access_level: AccessLevel,
    /// Rate limit configuration if any
    pub rate_limit: Option<RateLimit>,
    /// Request timeout override
    pub timeout: Option<Duration>,
    /// Whether retries are allowed for this endpoint
    pub allow_retries: bool,
    /// Additional endpoint-specific configuration
    pub metadata: HashMap<String, String>,
}

/// Registry of module endpoints
#[derive(Debug, Clone, Default)]
pub struct EndpointRegistry {
    endpoints: HashMap<String, EndpointConfig>,
}

impl EndpointRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            endpoints: HashMap::new(),
        }
    }

    /// Register a new endpoint configuration
    pub fn register(&mut self, config: EndpointConfig) {
        self.endpoints.insert(config.name.clone(), config);
    }

    /// Get configuration for an endpoint by name
    pub fn get(&self, name: &str) -> Option<&EndpointConfig> {
        self.endpoints.get(name)
    }

    /// Remove an endpoint configuration
    pub fn unregister(&mut self, name: &str) -> Option<EndpointConfig> {
        self.endpoints.remove(name)
    }

    /// List all registered endpoints
    pub fn list(&self) -> Vec<&EndpointConfig> {
        self.endpoints.values().collect()
    }

    /// Check if an endpoint exists
    pub fn exists(&self, name: &str) -> bool {
        self.endpoints.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_registry() {
        let mut registry = EndpointRegistry::new();
        
        let config = EndpointConfig {
            name: "test_endpoint".to_string(),
            path: "/test".to_string(),
            access_level: AccessLevel::Protected,
            rate_limit: Some(RateLimit {
                max_requests: 100,
                window_secs: 60,
            }),
            timeout: Some(Duration::from_secs(30)),
            allow_retries: true,
            metadata: HashMap::new(),
        };

        // Test registration
        registry.register(config.clone());
        assert!(registry.exists("test_endpoint"));

        // Test retrieval
        let retrieved = registry.get("test_endpoint").unwrap();
        assert_eq!(retrieved.name, "test_endpoint");
        assert_eq!(retrieved.access_level, AccessLevel::Protected);

        // Test listing
        let endpoints = registry.list();
        assert_eq!(endpoints.len(), 1);

        // Test unregistration
        let removed = registry.unregister("test_endpoint").unwrap();
        assert_eq!(removed.name, "test_endpoint");
        assert!(!registry.exists("test_endpoint"));
    }
}

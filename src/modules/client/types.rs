use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::time::Duration;
use std::clone::Clone;

/// Error information returned from module
#[derive(Debug, Clone, Deserialize)]
pub struct ModuleError {
    /// Error code
    pub code: u16,
    /// Error message
    pub message: String,
}

/// Response from module calls  
#[derive(Debug, Clone)]
pub struct ModuleResponse<T> where T: DeserializeOwned + 'static {
    /// Response data
    pub data: T,
    /// Error information if present
    pub error: Option<ModuleError>,
}

/// Configuration for the module client
#[derive(Debug, Clone)]
pub struct ModuleClientConfig {
    /// Base URL for the module server
    pub host: String,
    /// Port number
    pub port: u16,
    /// Request timeout
    pub timeout: Duration,
    /// Maximum number of retry attempts
    pub max_retries: u32,
}

impl Default for ModuleClientConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 5555,
            timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
}

/// Request parameters for module calls
#[derive(Debug, Clone, Serialize)]
pub struct ModuleRequest<T> where T: Clone + serde::Serialize {
    /// Target SS58 address
    pub target_key: String,
    /// Method-specific parameters
    pub params: T,
}

/// Custom error types for module client
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Network timeout after {0:?}")]
    Timeout(Duration),
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Maximum retries exceeded")]
    MaxRetriesExceeded,
    
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    #[error("Endpoint not found: {0}")]
    EndpointNotFound(String),
    
    #[error("Unknown error occurred")]
    Unknown,
    
    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid header")]
    InvalidHeader,
}
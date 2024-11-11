use std::time::Duration;
use serde_json::Value;
use crate::error::CommunexError;
use super::batch::BatchRequest;

pub struct RpcClient {
    url: String,
    timeout: Duration,
}

impl RpcClient {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            timeout: Duration::from_secs(30), // default timeout
        }
    }

    pub fn with_timeout(url: impl Into<String>, timeout: Duration) -> Self {
        Self {
            url: url.into(),
            timeout,
        }
    }

    pub async fn request(&self, method: &str, params: Value) -> Result<Value, CommunexError> {
        // Stub implementation that always fails
        Err(CommunexError::RpcError {
            code: -1,
            message: "Not implemented".to_string(),
        })
    }

    pub async fn batch_request(&self, batch: BatchRequest) -> Result<Vec<Value>, CommunexError> {
        // Stub implementation that always fails
        Err(CommunexError::RpcError {
            code: -1,
            message: "Not implemented".to_string(),
        })
    }
}

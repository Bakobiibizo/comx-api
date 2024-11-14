mod rpc_client;

pub use rpc_client::RpcClient;
use serde_json::{Value, json};
use std::time::Duration;
use crate::error::CommunexError;

#[derive(Debug, Clone)]
pub struct RpcClientConfig {
    /// Timeout for requests in seconds
    pub timeout: Duration,
    /// Maximum retries for failed requests
    pub max_retries: u32,
}

impl Default for RpcClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
}

impl RpcClientConfig {
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[derive(Debug)]
pub struct BatchRequest {
    pub requests: Vec<Value>,
}

impl BatchRequest {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    pub fn add_request(&mut self, method: &str, params: Value) {
        self.requests.push(json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": self.requests.len()
        }));
    }

    pub fn validate(&self) -> Result<(), CommunexError> {
        if self.requests.is_empty() {
            return Err(CommunexError::ValidationError(
                "Batch request cannot be empty".to_string()
            ));
        }

        if self.requests.len() > 100 {
            return Err(CommunexError::ValidationError(
                "Batch request cannot contain more than 100 requests".to_string()
            ));
        }

        for (i, request) in self.requests.iter().enumerate() {
            if !request.is_object() {
                return Err(CommunexError::ValidationError(
                    format!("Invalid request at index {}", i)
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct BatchResponse {
    pub successes: Vec<Value>,
    pub errors: Vec<RpcErrorDetail>,
}

#[derive(Debug)]
pub struct RpcErrorDetail {
    pub code: i32,
    pub message: String,
    pub request_id: Option<u32>,
}

impl RpcClient {
    pub async fn request_with_path(&self, path: &str, params: serde_json::Value) -> Result<serde_json::Value, CommunexError> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": path,
            "params": params
        });

        let response = self.send_request(path, &request).await?;
        
        if let Some(error) = response.get("error") {
            let code = error.get("code")
                .and_then(|c| c.as_i64())
                .unwrap_or(-32000);
            let message = error.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();
            
            return Err(CommunexError::RpcError { code: code as i32, message });
        }
        
        Ok(response.get("result").cloned().unwrap_or(json!({})))
    }

    async fn send_request(&self, path: &str, request: &serde_json::Value) -> Result<serde_json::Value, CommunexError> {
        let url = if self.url.ends_with('/') {
            format!("{}{}", self.url, path)
        } else {
            format!("{}/{}", self.url, path)
        };

        match self.client.post(&url)
            .json(request)
            .timeout(Duration::from_secs(5))
            .send()
            .await {
                Ok(response) => {
                    response.json().await.map_err(|e| {
                        CommunexError::MalformedResponse(e.to_string())
                    })
                },
                Err(e) => Err(CommunexError::ConnectionError(e.to_string()))
            }
    }
}


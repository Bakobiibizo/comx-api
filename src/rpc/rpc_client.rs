use std::time::Duration;
use reqwest::{Client as HttpClient, ClientBuilder};
use serde_json::{Value, json};
use crate::error::CommunexError;
use super::batch::BatchRequest;

pub struct RpcClient {
    url: String,
    client: HttpClient,
}

impl RpcClient {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: HttpClient::new(),
        }
    }

    pub fn with_timeout(url: impl Into<String>, timeout: Duration) -> Self {
        let client = ClientBuilder::new()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            url: url.into(),
            client,
        }
    }

    pub async fn request(&self, method: &str, params: Value) -> Result<Value, CommunexError> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let response = self.client
            .post(&self.url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| CommunexError::ConnectionError(e.to_string()))?;

        let status = response.status();
        let response_body: Value = response
            .json()
            .await
            .map_err(|e| CommunexError::ParseError(e.to_string()))?;

        if !status.is_success() {
            if let Some(error) = response_body.get("error") {
                return Err(CommunexError::RpcError {
                    code: error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1) as i32,
                    message: error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error").to_string(),
                });
            }
        }

        if let Some(error) = response_body.get("error") {
            return Err(CommunexError::RpcError {
                code: error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1) as i32,
                message: error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error").to_string(),
            });
        }

        Ok(response_body)
    }

    pub async fn batch_request(&self, batch: BatchRequest) -> Result<Vec<Value>, CommunexError> {
        let requests: Vec<Value> = batch.into_requests()
            .into_iter()
            .enumerate()
            .map(|(id, (method, params))| {
                json!({
                    "jsonrpc": "2.0",
                    "method": method,
                    "params": params,
                    "id": id + 1
                })
            })
            .collect();

        if requests.is_empty() {
            return Ok(vec![]);
        }

        let response = self.client
            .post(&self.url)
            .json(&requests)
            .send()
            .await
            .map_err(|e| CommunexError::ConnectionError(e.to_string()))?;

        let status = response.status();
        let response_body: Value = response
            .json()
            .await
            .map_err(|e| CommunexError::ParseError(e.to_string()))?;

        if !status.is_success() {
            return Err(CommunexError::ConnectionError(format!("HTTP {}", status)));
        }

        let responses = response_body.as_array()
            .ok_or_else(|| CommunexError::ParseError("Expected array response for batch request".to_string()))?;

        // Check for errors in individual responses
        for response in responses {
            if let Some(error) = response.get("error") {
                return Err(CommunexError::RpcError {
                    code: error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1) as i32,
                    message: error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error").to_string(),
                });
            }
        }

        Ok(responses.to_vec())
    }
}

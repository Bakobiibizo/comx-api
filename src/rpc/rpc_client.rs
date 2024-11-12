use crate::error::CommunexError;
use super::{BatchRequest, BatchResponse, RpcClientConfig, RpcErrorDetail};
use reqwest;
use serde_json::{json, Value};
use std::time::Duration;
use log::debug;
use futures::Future;

#[derive(Debug, Clone)]
pub struct RpcClient {
    url: String,
    client: reqwest::Client,
    config: RpcClientConfig,
}

impl RpcClient {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: reqwest::Client::new(),
            config: RpcClientConfig::default(),
        }
    }

    pub fn with_timeout(url: impl Into<String>, timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_default();

        Self {
            url: url.into(),
            client,
            config: RpcClientConfig::default(),
        }
    }

    pub fn new_with_config(url: impl Into<String>, config: RpcClientConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap_or_default();

        Self {
            url: url.into(),
            client,
            config,
        }
    }

    async fn handle_rpc_response(&self, value: Value) -> Result<Value, CommunexError> {
        if let Some(error) = value.get("error") {
            let code = error.get("code")
                .and_then(|c| c.as_i64())
                .map(|c| c as i32)
                .unwrap_or(-32603);
            let message = error.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();
            
            return Err(CommunexError::RpcError { code, message });
        }

        value.get("result")
            .cloned()
            .ok_or_else(|| CommunexError::ParseError("Missing result field".to_string()))
    }

    pub async fn request(&self, method: &str, params: Value) -> Result<Value, CommunexError> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        self.execute_with_retry(|| async {
            let response = self.client
                .post(&self.url)
                .json(&request)
                .send()
                .await
                .map_err(|e| CommunexError::ConnectionError(e.to_string()))?;

            let value = response
                .json::<Value>()
                .await
                .map_err(|e| CommunexError::ParseError(e.to_string()))?;

            self.handle_rpc_response(value).await
        }).await
    }

    pub async fn batch_request(&self, batch: BatchRequest) -> Result<BatchResponse, CommunexError> {
        let response = self.client.post(&self.url)
            .json(&batch.requests)
            .send()
            .await
            .map_err(|e| CommunexError::ConnectionError(e.to_string()))?
            .json::<Vec<Value>>()
            .await
            .map_err(|e| CommunexError::ParseError(e.to_string()))?;

        let mut successes = Vec::new();
        let mut errors = Vec::new();

        for resp in response {
            if let Some(error) = resp.get("error") {
                let code = error.get("code")
                    .and_then(|c| c.as_i64())
                    .map(|c| c as i32)
                    .unwrap_or(-32603);
                let message = error.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();
                let request_id = resp.get("id")
                    .and_then(|id| id.as_u64())
                    .map(|id| id as u32);
                
                errors.push(RpcErrorDetail { code, message, request_id });
            } else if let Some(result) = resp.get("result") {
                successes.push(result.clone());
            }
        }

        Ok(BatchResponse {
            successes,
            errors,
        })
    }

    pub async fn batch_balance_request(&self, addresses: &[&str]) -> Result<BatchResponse, CommunexError> {
        let mut batch = BatchRequest::new();
        
        for address in addresses {
            batch.add_request(
                "query_balance",
                json!({
                    "address": address
                })
            );
        }

        self.batch_request(batch).await
    }

    async fn handle_batch_response(&self, responses: Vec<Value>) -> Result<Vec<Value>, CommunexError> {
        let mut results = Vec::new();
        for response in responses {
            if let Some(error) = response.get("error") {
                let code = error.get("code")
                    .and_then(|c| c.as_i64())
                    .map(|c| c as i32)
                    .unwrap_or(-32603);
                let message = error.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();
                
                return Err(CommunexError::RpcError { code, message });
            }

            if let Some(result) = response.get("result") {
                results.push(result.clone());
            } else {
                return Err(CommunexError::ParseError("Missing result field in batch response".to_string()));
            }
        }
        Ok(results)
    }

    pub async fn send_batch_request(&self, batch: BatchRequest) -> Result<Vec<Value>, CommunexError> {
        let mut requests = Vec::new();
        for request in batch.requests.iter() {
            requests.push(json!({
                "jsonrpc": "2.0",
                "method": request["method"],
                "params": request["params"],
                "id": request["id"]
            }));
        }   

        if requests.is_empty() {
            return Ok(vec![]);
        }

        let response = self.client
            .post(&self.url)
            .json(&requests)
            .send()
            .await
            .map_err(|e| CommunexError::ConnectionError(e.to_string()))?;

        let response_body: Value = response
            .json()
            .await
            .map_err(|e| CommunexError::ParseError(e.to_string()))?;

        let responses = response_body.as_array()
            .ok_or_else(|| CommunexError::ParseError("Expected array response for batch request".to_string()))?;

        self.handle_batch_response(responses.to_vec()).await
    }

    pub async fn execute_with_retry<T, F, Fut>(&self, f: F) -> Result<T, CommunexError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, CommunexError>>,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.config.max_retries {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e);
                    if attempts < self.config.max_retries {
                        debug!("Request failed, retrying ({}/{})", attempts, self.config.max_retries);
                        tokio::time::sleep(Duration::from_millis(100 * 2u64.pow(attempts))).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| CommunexError::ConnectionError(
            "Maximum retries exceeded".to_string()
        )))
    }
}

use std::time::Duration;
use reqwest::{Client as HttpClient, ClientBuilder};
use serde_json::{Value, json};
use crate::error::{CommunexError, RpcErrorDetail};
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

    async fn handle_rpc_response(&self, response: Value) -> Result<Value, CommunexError> {
        if let Some(error) = response.get("error") {
            let code = error.get("code")
                .and_then(|c| c.as_i64())
                .unwrap_or(-32603) as i32;
            let message = error.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();
            
            return Err(CommunexError::RpcError { code, message });
        }

        response.get("result")
            .cloned()
            .ok_or_else(|| CommunexError::MalformedResponse("Missing 'result' field".to_string()))
    }

    async fn handle_batch_response(&self, responses: Vec<Value>) -> Result<Vec<Value>, CommunexError> {
        let mut errors = Vec::new();
        let mut results = Vec::new();

        for (idx, response) in responses.into_iter().enumerate() {
            match self.handle_rpc_response(response.clone()).await {
                Ok(result) => results.push(result),
                Err(CommunexError::RpcError { code, message }) => {
                    errors.push(RpcErrorDetail {
                        code,
                        message,
                        request_id: response.get("id").and_then(|id| id.as_u64()),
                    });
                }
                Err(e) => errors.push(RpcErrorDetail {
                    code: -32603,
                    message: format!("Request {}: {}", idx, e),
                    request_id: Some(idx as u64),
                }),
            }
        }

        if !errors.is_empty() {
            Err(CommunexError::BatchRpcError(errors))
        } else {
            Ok(results)
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

        let response_body: Value = response
            .json()
            .await
            .map_err(|e| CommunexError::ParseError(e.to_string()))?;

        self.handle_rpc_response(response_body).await
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

        let response_body: Value = response
            .json()
            .await
            .map_err(|e| CommunexError::ParseError(e.to_string()))?;

        self.handle_batch_response(response_body.as_array().unwrap().to_vec()).await
    }
}

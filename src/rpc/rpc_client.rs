use std::time::Duration;
use core::clone::Clone;
use reqwest::{Client as HttpClient, ClientBuilder};
use serde_json::{Value, json};
use crate::error::{CommunexError, RpcErrorDetail};
use super::batch::BatchRequest;


#[derive(Debug, Clone)]
pub struct RpcClient {
    url: String,
    client: HttpClient,
}

#[derive(Debug)]
pub struct BatchResponse {
    pub successes: Vec<Value>,
    pub errors: Vec<RpcErrorDetail>,
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
        let mut results = Vec::new();

        for response in responses {
            // For error responses, include the error object directly
            if response.get("error").is_some() {
                results.push(response);
                continue;
            }

            // For successful responses, include the entire response
            results.push(response);
        }

        Ok(results)
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

    pub async fn batch_request(&self, batch: BatchRequest) -> Result<BatchResponse, CommunexError> {
        let responses = self.send_batch_request(batch).await?;
        let mut successes = Vec::new();
        let mut errors = Vec::new();

        for resp in responses {
            if let Some(error) = resp.get("error") {
                errors.push(RpcErrorDetail {
                    code: error["code"].as_i64().unwrap_or(0) as i32,
                    message: error["message"].as_str().unwrap_or("").to_string(),
                    request_id: resp["id"].as_u64(),
                });
            } else if let Some(result) = resp.get("result") {
                successes.push(result.clone());
            }
        }

        Ok(BatchResponse { successes, errors })
    }

    async fn send_batch_request(&self, batch: BatchRequest) -> Result<Vec<Value>, CommunexError> {
        let mut requests = Vec::new();
        for (method, params) in batch.into_requests().iter() {
            requests.push(json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
                "id": 1
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

        self.handle_batch_response(response_body.as_array().unwrap().to_vec()).await
    }
}

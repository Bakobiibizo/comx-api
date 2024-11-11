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

    pub async fn batch_request(&self, batch: BatchRequest) -> Result<Vec<Value>, CommunexError> {
        let responses = self.send_batch_request(batch).await?;
        
        // Check if all responses are errors
        let errors: Vec<RpcErrorDetail> = responses.iter()
            .filter_map(|resp| {
                if resp.get("error").is_some() {
                    Some(RpcErrorDetail {
                        code: resp["error"]["code"].as_i64().unwrap_or(0) as i32,
                        message: resp["error"]["message"].as_str().unwrap_or("").to_string(),
                        request_id: resp["id"].as_u64(),
                    })
                } else {
                    None
                }
            })
            .collect();

        if !errors.is_empty() {
            return Err(CommunexError::BatchRpcError(errors));
        }

        let results: Vec<Value> = responses.iter()
            .filter_map(|resp| resp.get("result"))
            .cloned()
            .collect();

        Ok(results)
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

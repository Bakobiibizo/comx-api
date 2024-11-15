use crate::crypto::KeyPair;
use reqwest::{Client as HttpClient, header};
use std::fmt::Display;
use std::time::Duration;
use tokio::time::timeout;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use self::types::{ClientError, ModuleClientConfig, ModuleRequest, ModuleResponse};

pub mod types;

/// Client for communicating with module servers
pub struct ModuleClient {
    config: ModuleClientConfig,
    http_client: HttpClient,
    keypair: KeyPair,
}

impl ModuleClient {
    /// Create a new module client with default configuration
    pub fn new(keypair: KeyPair) -> Self {
        Self::with_config(ModuleClientConfig::default(), keypair)
    }

    /// Create a new module client with custom configuration
    pub fn with_config(config: ModuleClientConfig, keypair: KeyPair) -> Self {
        let http_client = HttpClient::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
            keypair,
        }
    }

    /// Call a module method
    pub async fn call<T, R>(&self, method: &str, target_key: &str, params: T) -> Result<R, ClientError>
    where
        T: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let timestamp = Utc::now();
        let request = self.build_request(method, target_key, params, timestamp)?;
        
        for retry in 0..self.config.max_retries {
            match self.execute_request(&request, timestamp).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if retry == self.config.max_retries - 1 || !self.should_retry(&e) {
                        return Err(e);
                    }
                    tokio::time::sleep(self.calculate_backoff(retry)).await;
                }
            }
        }
        
        Err(ClientError::MaxRetriesExceeded)
    }

    fn build_request<T: serde::Serialize>(
        &self,
        method: &str,
        target_key: &str,
        params: T,
        timestamp: DateTime<Utc>,
    ) -> Result<(String, header::HeaderMap, ModuleRequest<T>), ClientError> {
        let request = ModuleRequest {
            target_key: target_key.to_string(),
            params,
        };

        let url = format!(
            "http://{}:{}/{}",
            self.config.host, self.config.port, method
        );

        let signature = self.sign_request(&request, timestamp)?;
        let headers = self.build_headers(signature, timestamp)?;

        Ok((url, headers, request))
    }

    fn should_retry(&self, error: &ClientError) -> bool {
        matches!(
            error,
            ClientError::Timeout(_) | 
            ClientError::RateLimitExceeded |
            ClientError::ServerError(_)
        )
    }

    fn calculate_backoff(&self, retry: u32) -> Duration {
        Duration::from_millis(100 * 2u64.pow(retry))
    }

    fn build_headers(
        &self,
        signature: String,
        timestamp: DateTime<Utc>,
    ) -> Result<header::HeaderMap, ClientError> {
        let mut headers = header::HeaderMap::new();
        
        // Add required headers with proper error handling
        headers.insert("X-Signature", signature.parse().map_err(|_| ClientError::InvalidHeader)?);
        headers.insert("X-Key", self.keypair.public_key().to_string().parse().map_err(|_| ClientError::InvalidHeader)?);
        headers.insert("X-Crypto", "sr25519".parse().map_err(|_| ClientError::InvalidHeader)?);
        headers.insert("X-Timestamp", timestamp.to_rfc3339().parse().map_err(|_| ClientError::InvalidHeader)?);

        Ok(headers)
    }

    // Helper method to sign requests
    fn sign_request<T: serde::Serialize>(&self, request: &ModuleRequest<T>) -> Result<String, ClientError> {
        let message = serde_json::to_string(request)
            .map_err(|e| ClientError::RequestFailed(e.to_string()))?;
        Ok(self.keypair.sign(message.as_bytes()))
    }
}

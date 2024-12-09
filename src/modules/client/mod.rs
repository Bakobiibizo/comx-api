mod types;
mod endpoint;

pub use types::{ModuleClientConfig, ClientError, ModuleRequest, ModuleResponse};
pub use endpoint::{EndpointConfig, EndpointRegistry, AccessLevel, RateLimit};

use crate::crypto::KeyPair;
use reqwest::{Client as HttpClient, header};
use serde::Serialize;
use std::time::Duration;
use chrono::{DateTime, Utc};
use hex;

/// Client for communicating with module servers
pub struct ModuleClient {
    pub config: ModuleClientConfig,
    pub http_client: HttpClient,
    pub keypair: KeyPair,
    pub endpoint_registry: EndpointRegistry,
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
            endpoint_registry: EndpointRegistry::new(),
        }
    }

    /// Register a new endpoint configuration
    pub fn register_endpoint(&mut self, config: EndpointConfig) {
        self.endpoint_registry.register(config);
    }

    /// Get endpoint configuration by name
    pub fn get_endpoint(&self, name: &str) -> Option<&EndpointConfig> {
        self.endpoint_registry.get(name)
    }

    /// Call a module method
    pub async fn call<T, R>(&self, method: &str, target_key: &str, params: T) -> Result<R, ClientError>
    where
        T: serde::Serialize + Clone,
        R: serde::de::DeserializeOwned,
    {
        // Get endpoint configuration if it exists
        let endpoint_config = self.endpoint_registry.get(method);
        
        // Validate access level if endpoint is configured
        if let Some(config) = endpoint_config {
            match config.access_level {
                AccessLevel::Private | AccessLevel::Protected => {
                    // Additional access validation could be added here
                }
                AccessLevel::Public => {}
            }
        }

        let timestamp = Utc::now();
        let request = self.build_request(method, target_key, params, timestamp)?;
        
        let mut last_error = None;
        let max_retries = endpoint_config
            .map(|c| if c.allow_retries { self.config.max_retries } else { 0 })
            .unwrap_or(self.config.max_retries);

        for retry in 0..=max_retries {
            match self.execute_request(&method, request.0.clone(), request.1.clone(), request.2.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if retry == max_retries || !self.should_retry(&e) {
                        return Err(e);
                    }
                    last_error = Some(e);
                    tokio::time::sleep(self.calculate_backoff(retry)).await;
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| ClientError::Unknown))
    }

    async fn execute_request<T: Serialize + Clone, R>(
        &self,
        method: &str,
        url: String,
        headers: header::HeaderMap,
        request: ModuleRequest<T>,
    ) -> Result<R, ClientError>
    where
        R: serde::de::DeserializeOwned, T: Serialize,
    {
        let response = self.http_client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| match e.is_timeout() {
                true => ClientError::Timeout(self.config.timeout),
                false => ClientError::RequestFailed(e.to_string()),
            })?;

        match response.status() {
            reqwest::StatusCode::OK => {
                response.json::<R>().await.map_err(|e| ClientError::RequestFailed(e.to_string()))
            }
            reqwest::StatusCode::UNAUTHORIZED => Err(ClientError::Unauthorized),
            reqwest::StatusCode::TOO_MANY_REQUESTS => Err(ClientError::RateLimitExceeded),
            reqwest::StatusCode::NOT_FOUND => Err(ClientError::MethodNotFound(method.to_string())),
            status => Err(ClientError::ServerError(status.to_string())),
        }
    }

    fn build_request<T>(
        &self,
        method: &str,
        target_key: &str,
        params: T,
        timestamp: DateTime<Utc>,
    ) -> Result<(String, header::HeaderMap, ModuleRequest<T>), ClientError>
    where
        T: serde::Serialize + Clone,
    {
        let request = ModuleRequest {
            target_key: target_key.to_string(),
            params,
        };

        // Handle URLs with and without port numbers
        let url = if self.config.port == 0 {
            format!("{}/{}", self.config.host.trim_end_matches('/'), method)
        } else {
            format!(
                "{}:{}/{}",
                self.config.host.trim_end_matches('/'),
                self.config.port,
                method
            )
        };

        let message = serde_json::to_string(&request)
            .map_err(|e| ClientError::SerializationError(e.to_string()))?;
        let signature = self.sign_request(&message)?;
        let headers = self.build_headers(signature, timestamp)?;

        Ok((url, headers, request))
    }

    fn should_retry(&self, error: &ClientError) -> bool {
        matches!(
            error,
            ClientError::Timeout(_) | 
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
        
        headers.insert(
            header::CONTENT_TYPE,
            "application/json".parse().map_err(|_| ClientError::InvalidHeader)?
        );
        headers.insert(
            "X-Signature",
            signature.parse().map_err(|_| ClientError::InvalidHeader)?
        );
        headers.insert(
            "X-Key",
            self.keypair.public_key_hex().parse().map_err(|_| ClientError::InvalidHeader)?
        );
        headers.insert(
            "X-Timestamp",
            timestamp.to_rfc3339().parse().map_err(|_| ClientError::InvalidHeader)?
        );

        Ok(headers)
    }

    fn sign_request(&self, message: &str) -> Result<String, ClientError> {
        let signature = self.keypair.sign(message.as_bytes());
        Ok(hex::encode(signature))
    }
}

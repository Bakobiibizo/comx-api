use std::sync::Arc;
use serde_json::json;
use crate::{
    rpc::RpcClient,
    types::{Address, Balance},
    error::CommunexError,
};
use super::QueryMapConfig;
use tokio::time::Duration;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub struct QueryMap {
    client: Arc<RpcClient>,
    config: QueryMapConfig,
    refresh_count: AtomicU64,
}

impl QueryMap {
    pub fn new(client: RpcClient, config: QueryMapConfig) -> Result<Self, CommunexError> {
        // Validate minimum refresh interval (e.g., 1 second)
        if config.refresh_interval < Duration::from_secs(1) {
            return Err(CommunexError::CommunexError(
                "Refresh interval must be at least 1 second".to_string()
            ));
        }

        // Validate cache duration is longer than refresh interval
        if config.cache_duration <= config.refresh_interval {
            return Err(CommunexError::CommunexError(
                "Cache duration must be longer than refresh interval".to_string()
            ));
        }

        Ok(Self {
            client: Arc::new(client),
            config,
            refresh_count: AtomicU64::new(0),
        })
    }

    pub async fn get_balance(&self, address: &str) -> Result<Balance, CommunexError> {
        self.refresh_count.fetch_add(1, Ordering::Relaxed);
        let params = json!({
            "address": address
        });

        let response = self.client
            .request("query_balance", params)
            .await?;

        // Convert response to Balance type
        serde_json::from_value(response)
            .map_err(|e| CommunexError::ParseError(e.to_string()))
    }

    pub async fn get_balances(&self, addresses: &[&str]) -> Result<Vec<Balance>, CommunexError> {
        let mut batch = crate::rpc::BatchRequest::new();
        
        for address in addresses {
            batch.add_request(
                "query_balance",
                json!({
                    "address": address
                })
            );
        }

        let response = self.client.batch_request(batch).await?;
        
        // Convert successful responses to Balance objects
        response.successes
            .into_iter()
            .map(|value| {
                serde_json::from_value(value)
                    .map_err(|e| CommunexError::ParseError(e.to_string()))
            })
            .collect()
    }

    pub async fn get_stake_from(&self, address: &str) -> Result<Vec<Address>, CommunexError> {
        let params = json!({
            "address": address
        });

        let response = self.client
            .request("query_stakefrom", params)
            .await?;

        // Extract stake_from array from response
        let stake_from = response.get("stake_from")
            .ok_or_else(|| CommunexError::ParseError("Missing stake_from field".to_string()))?;

        let addresses: Vec<String> = serde_json::from_value(stake_from.clone())
            .map_err(|e| CommunexError::ParseError(e.to_string()))?;

        // Convert each string to an Address type
        addresses.into_iter()
            .map(|addr| Address::new(&addr))
            .collect()
    }

    pub async fn get_stake_to(&self, address: &str) -> Result<Vec<Address>, CommunexError> {
        let params = json!({
            "address": address
        });

        let response = self.client
            .request("query_staketo", params)
            .await?;

        // Extract stake_to array from response
        let stake_to = response.get("stake_to")
            .ok_or_else(|| CommunexError::ParseError("Missing stake_to field".to_string()))?;

        let addresses: Vec<String> = serde_json::from_value(stake_to.clone())
            .map_err(|e| CommunexError::ParseError(e.to_string()))?;

        // Convert each string to an Address type
        addresses.into_iter()
            .map(|addr| Address::new(&addr))
            .collect()
    }

    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            refresh_count: self.refresh_count.load(Ordering::Relaxed),
        }
    }
}

pub struct CacheStats {
    pub refresh_count: u64,
} 
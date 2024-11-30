use std::sync::Arc;
use serde_json::json;
use crate::{
    rpc::RpcClient,
    types::{Address, Balance},
    error::CommunexError,
};
use super::QueryMapConfig;
use std::sync::atomic::{AtomicU64, Ordering};

/// QueryMap provides high-level access to blockchain state queries with caching support.
/// It automatically handles RPC communication and response parsing.
#[derive(Debug)]
pub struct QueryMap {
    client: Arc<RpcClient>,
    #[allow(dead_code)]  // Used for configuration but not read directly
    config: QueryMapConfig,
    refresh_count: AtomicU64,
}

impl QueryMap {
    /// Creates a new QueryMap instance with the given RPC client and configuration.
    /// 
    /// # Arguments
    /// * `client` - The RPC client to use for queries
    /// * `config` - Configuration for cache behavior
    /// 
    /// # Returns
    /// * `Result<QueryMap, CommunexError>` - New QueryMap instance or error if config is invalid
    pub fn new(client: RpcClient, config: QueryMapConfig) -> Result<Self, CommunexError> {
        config.validate()?;
        
        Ok(Self {
            client: Arc::new(client),
            config,
            refresh_count: AtomicU64::new(0),
        })
    }

    /// Retrieves the balance for a single address.
    /// 
    /// # Arguments
    /// * `address` - The address to query
    /// 
    /// # Returns
    /// * `Result<Balance, CommunexError>` - Balance information or error
    pub async fn get_balance(&self, address: &str) -> Result<Balance, CommunexError> {
        debug!("Querying balance for address: {}", address);
        self.refresh_count.fetch_add(1, Ordering::Relaxed);
        
        let params = json!({
            "address": address
        });

        let response = self.client
            .request("query_balance", params)
            .await?;

        trace!("Received balance response: {:?}", response);
        
        // Convert response to Balance type with better error context
        serde_json::from_value(response)
            .map_err(|e| {
                error!("Failed to parse balance response: {}", e);
                CommunexError::ParseError(format!("Failed to parse balance response: {}", e))
            })
    }

    pub async fn get_balances(&self, addresses: &[&str]) -> Result<Vec<Balance>, CommunexError> {
        if addresses.is_empty() {
            return Ok(Vec::new());
        }

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
                    .map_err(|e| CommunexError::ParseError(
                        format!("Failed to parse balance in batch response: {}", e)
                    ))
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

        let stake_from = response.get("stake_from")
            .ok_or_else(|| CommunexError::ParseError(
                "Response missing 'stake_from' field".to_string()
            ))?;

        let addresses: Vec<String> = serde_json::from_value(stake_from.clone())
            .map_err(|e| CommunexError::ParseError(
                format!("Failed to parse stake_from addresses: {}", e)
            ))?;

        addresses.into_iter()
            .map(|addr| Address::new(&addr))
            .collect::<Result<Vec<_>, _>>()
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
            // Relaxed ordering is sufficient for metrics that don't require
            // synchronization with other operations
            refresh_count: self.refresh_count.load(Ordering::Relaxed),
        }
    }
}

pub struct CacheStats {
    pub refresh_count: u64,
} 
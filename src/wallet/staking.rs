use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::error::CommunexError;
use crate::wallet::{WalletClient, TransactionState};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeRequest {
    pub from: String,
    pub amount: u64,
    pub denom: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnstakeRequest {
    pub from: String,
    pub amount: Option<u64>,  // None means unstake all
    pub denom: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingInfo {
    pub address: String,
    pub total_staked: u64,
    pub rewards_available: u64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub last_claim_time: DateTime<Utc>,
    pub denom: String,
}

impl WalletClient {
    pub async fn stake(&self, request: StakeRequest) -> Result<TransactionState, CommunexError> {
        if !request.from.starts_with("cmx1") {
            return Err(CommunexError::RpcError {
                code: -32001,
                message: "Invalid address".into(),
            });
        }

        let params = json!({
            "from": request.from,
            "amount": request.amount,
            "denom": request.denom,
        });

        let response = self.rpc_client.request_with_path("staking/stake", params).await?;
        
        // Get transaction hash from response
        let tx_hash = response.get("hash")
            .and_then(|v| v.as_str())
            .ok_or(CommunexError::MalformedResponse("Missing transaction hash".into()))?;

        // Wait for transaction confirmation
        self.wait_for_transaction(tx_hash, std::time::Duration::from_secs(30)).await
    }

    pub async fn unstake(&self, request: UnstakeRequest) -> Result<TransactionState, CommunexError> {
        if !request.from.starts_with("cmx1") {
            return Err(CommunexError::RpcError {
                code: -32001,
                message: "Invalid address".into(),
            });
        }

        let params = json!({
            "from": request.from,
            "amount": request.amount,
            "denom": request.denom,
        });

        let response = self.rpc_client.request_with_path("staking/unstake", params).await?;
        
        let tx_hash = response.get("hash")
            .and_then(|v| v.as_str())
            .ok_or(CommunexError::MalformedResponse("Missing transaction hash".into()))?;

        self.wait_for_transaction(tx_hash, std::time::Duration::from_secs(30)).await
    }

    pub async fn claim_rewards(&self, address: &str) -> Result<TransactionState, CommunexError> {
        if !address.starts_with("cmx1") {
            return Err(CommunexError::RpcError {
                code: -32001,
                message: "Invalid address".into(),
            });
        }

        let params = json!({
            "address": address,
        });

        let response = self.rpc_client.request_with_path("staking/claim", params).await?;
        
        let tx_hash = response.get("hash")
            .and_then(|v| v.as_str())
            .ok_or(CommunexError::MalformedResponse("Missing transaction hash".into()))?;

        self.wait_for_transaction(tx_hash, std::time::Duration::from_secs(30)).await
    }

    pub async fn get_staking_info(&self, address: &str) -> Result<StakingInfo, CommunexError> {
        if !address.starts_with("cmx1") {
            return Err(CommunexError::RpcError {
                code: -32001,
                message: "Invalid address".into(),
            });
        }

        let params = json!({
            "address": address,
        });

        match self.rpc_client.request_with_path("staking/info", params).await {
            Ok(response) => {
                Ok(StakingInfo {
                    address: address.to_string(),
                    total_staked: response.get("total_staked")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    rewards_available: response.get("rewards_available")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    last_claim_time: response.get("last_claim_time")
                        .and_then(|v| v.as_i64())
                        .map(|ts| DateTime::<Utc>::from_timestamp(ts, 0))
                        .flatten()
                        .unwrap_or_else(|| Utc::now()),
                    denom: response.get("denom")
                        .and_then(|v| v.as_str())
                        .unwrap_or("COMAI")
                        .to_string(),
                })
            },
            Err(e) => Err(e)
        }
    }
} 
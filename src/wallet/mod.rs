use crate::{CommunexError, rpc::RpcClient};
use serde::{Serialize, Deserialize};
use serde_json::json;
use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};
pub mod staking;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub denom: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResponse {
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub free: u64,
    pub reserved: u64,
    pub misc_frozen: u64,
    pub fee_frozen: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionHistory {
    pub hash: String,
    pub block_num: u64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub denom: String,
    pub state: TransactionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Success,
    Failed,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionState {
    pub hash: String,
    pub block_num: Option<u64>,
    pub confirmations: u64,
    pub state: Txstate,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Txstate {
    Pending,
    Success,
    Failed,
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTransferResult {
    pub batch_id: String,
    pub transactions: Vec<BatchTransactionStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTransactionStatus {
    pub hash: String,
    pub status: TransactionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub struct WalletClient {
    pub rpc_client: RpcClient,
}

// Constants for validation
const MAX_BATCH_SIZE: usize = 100;
const VALID_DENOMS: [&str; 1] = ["COMAI"];
const MIN_AMOUNT: u64 = 1;

impl WalletClient {
    pub fn new(url: &str) -> Self {
        Self {
            rpc_client: RpcClient::new(url),
        }
    }

    pub fn with_timeout(url: &str, timeout: Duration) -> Self {
        Self {
            rpc_client: RpcClient::with_timeout(url, timeout),
        }
    }

    pub async fn transfer(&self, request: TransferRequest) -> Result<TransferResponse, CommunexError> {
        // Validate request before making RPC call
        if request.amount == 0 {
            return Err(CommunexError::RpcError {
                code: -32002,
                message: "Amount must be greater than zero".into(),
            });
        }

        if !request.denom.eq("COMAI") {
            return Err(CommunexError::RpcError {
                code: -32003,
                message: "Unsupported denomination".into(),
            });
        }

        if !request.from.starts_with("cmx1") {
            return Err(CommunexError::RpcError {
                code: -32001,
                message: "Invalid address".into(),
            });
        }

        // Prepare RPC request
        let params = json!({
            "from": request.from,
            "to": request.to,
            "amount": request.amount.to_string(),
            "denom": request.denom,
        });

        // Send RPC request
        match self.rpc_client.request_with_path("transfer", params).await {
            Ok(response) => {
                Ok(TransferResponse {
                    state: response.get("state")
                        .and_then(|s| s.as_str())
                        .unwrap_or("success")
                        .to_string(),
                })
            },
            Err(CommunexError::RpcError { code, message }) => {
                match code {
                    -32000 => Err(CommunexError::RpcError {
                        code: -32000,
                        message: "Insufficient funds".into()
                    }),
                    _ => Err(CommunexError::RpcError { code, message })
                }
            },
            Err(_) => {
                Err(CommunexError::ConnectionError("Failed to connect to server".into()))
            }
        }
    }

    pub async fn get_free_balance(&self, address: &str) -> Result<u64, CommunexError> {
        if !address.starts_with("cmx1") {
            return Err(CommunexError::RpcError {
                code: -32001,
                message: "Invalid address".into(),
            });
        }

        let params = json!({
            "address": address,
        });

        match self.rpc_client.request_with_path("balance/free", params).await {
            Ok(response) => {
                Ok(response.get("free")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0))
            },
            Err(e) => Err(e)
        }
    }

    pub async fn get_all_balances(&self, address: &str) -> Result<BalanceInfo, CommunexError> {
        if !address.starts_with("cmx1") {
            return Err(CommunexError::RpcError {
                code: -32001,
                message: "Invalid address".into(),
            });
        }

        let params = json!({
            "address": address,
        });

        match self.rpc_client.request_with_path("balance/all", params).await {
            Ok(response) => {
                Ok(BalanceInfo {
                    free: response.get("free").and_then(|v| v.as_u64()).unwrap_or(0),
                    reserved: response.get("reserved").and_then(|v| v.as_u64()).unwrap_or(0),
                    misc_frozen: response.get("miscFrozen").and_then(|v| v.as_u64()).unwrap_or(0),
                    fee_frozen: response.get("feeFrozen").and_then(|v| v.as_u64()).unwrap_or(0),
                })
            },
            Err(e) => Err(e)
        }
    }

    pub async fn get_staked_balance(&self, address: &str) -> Result<u64, CommunexError> {
        if !address.starts_with("cmx1") {
            return Err(CommunexError::RpcError {
                code: -32001,
                message: "Invalid address".into(),
            });
        }

        let params = json!({
            "address": address,
        });

        match self.rpc_client.request_with_path("balance/staked", params).await {
            Ok(response) => {
                Ok(response.get("staked")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0))
            },
            Err(e) => Err(e)
        }
    }

    pub async fn get_transaction_history(&self, address: &str) -> Result<Vec<TransactionHistory>, CommunexError> {
        if !address.starts_with("cmx1") {
            return Err(CommunexError::RpcError {
                code: -32001,
                message: "Invalid address".into(),
            });
        }

        let params = json!({
            "address": address,
        });

        match self.rpc_client.request_with_path("transaction/history", params).await {
            Ok(response) => {
                let transactions = response.get("transactions")
                    .and_then(|v| v.as_array())
                    .ok_or(CommunexError::MalformedResponse("Missing transactions array".into()))?;

                transactions.iter()
                    .map(|tx| {
                        Ok(TransactionHistory {
                            hash: tx.get("hash")
                                .and_then(|v| v.as_str())
                                .ok_or(CommunexError::MalformedResponse("Missing hash".into()))?
                                .to_string(),
                            block_num: tx.get("block_num")
                                .and_then(|v| v.as_u64())
                                .ok_or(CommunexError::MalformedResponse("Missing block number".into()))?,
                            timestamp: tx.get("timestamp")
                                .and_then(|v| v.as_i64())
                                .map(|ts| DateTime::<Utc>::from_timestamp(ts, 0))
                                .flatten()
                                .ok_or(CommunexError::MalformedResponse("Invalid timestamp".into()))?,
                            from: tx.get("from")
                                .and_then(|v| v.as_str())
                                .ok_or(CommunexError::MalformedResponse("Missing from address".into()))?
                                .to_string(),
                            to: tx.get("to")
                                .and_then(|v| v.as_str())
                                .ok_or(CommunexError::MalformedResponse("Missing to address".into()))?
                                .to_string(),
                            amount: tx.get("amount")
                                .and_then(|v| v.as_u64())
                                .ok_or(CommunexError::MalformedResponse("Missing amount".into()))?,
                            denom: tx.get("denom")
                                .and_then(|v| v.as_str())
                                .ok_or(CommunexError::MalformedResponse("Missing denomination".into()))?
                                .to_string(),
                            state: match tx.get("state").and_then(|v| v.as_str()) {
                                Some("success") => TransactionStatus::Success,
                                Some("failed") => TransactionStatus::Failed,
                                Some("pending") => TransactionStatus::Pending,
                                _ => TransactionStatus::Failed,
                            },
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
            },
            Err(e) => Err(e)
        }
    }

    pub async fn get_transaction_state(&self, tx_hash: &str) -> Result<TransactionState, CommunexError> {
        let params = json!({
            "hash": tx_hash,
        });

        match self.rpc_client.request_with_path("transaction/state", params).await {
            Ok(response) => {
                Ok(TransactionState {
                    hash: tx_hash.to_string(),
                    block_num: response.get("block_num")
                        .and_then(|v| v.as_u64()),
                    confirmations: response.get("confirmations")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    state: match response.get("state").and_then(|v| v.as_str()) {
                        Some("success") => Txstate::Success,
                        Some("failed") => Txstate::Failed,
                        Some("pending") => Txstate::Pending,
                        _ => Txstate::NotFound,
                    },
                    timestamp: response.get("timestamp")
                        .and_then(|v| v.as_i64())
                        .map(|ts| DateTime::<Utc>::from_timestamp(ts, 0))
                        .flatten()
                        .unwrap_or_else(|| Utc::now()),
                    error: response.get("error")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                })
            },
            Err(e) => Err(e)
        }
    }

    pub async fn wait_for_transaction(&self, tx_hash: &str, timeout: Duration) -> Result<TransactionState, CommunexError> {
        let start_time = Instant::now();
        
        while start_time.elapsed() < timeout {
            let state = self.get_transaction_state(tx_hash).await?;
            
            match state.state {
                Txstate::Success | Txstate::Failed => return Ok(state),
                _ => {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            }
        }
        
        Err(CommunexError::RequestTimeout("Transaction wait timeout".into()))
    }

    pub async fn batch_transfer(&self, transfers: Vec<TransferRequest>) -> Result<BatchTransferResult, CommunexError> {
        // Validate batch size
        if transfers.is_empty() {
            return Err(CommunexError::ValidationError("Transfer list cannot be empty".into()));
        }
        if transfers.len() > MAX_BATCH_SIZE {
            return Err(CommunexError::ValidationError(
                format!("Batch size exceeds maximum limit of {}", MAX_BATCH_SIZE)
            ));
        }

        // Validate each transfer
        for transfer in transfers.iter() {
            self.validate_transfer(transfer)?;
        }

        let params = json!({
            "transfers": transfers
        });

        let response = self.rpc_client
            .request("batch_transfer", params)
            .await
            .map_err(|e| match e {
                CommunexError::RequestTimeout(_) => 
                    CommunexError::RequestTimeout("Batch transfer request timed out".into()),
                _ => e
            })?;

        serde_json::from_value(response)
            .map_err(|e| CommunexError::ParseError(
                format!("Failed to parse batch transfer response: {}", e)
            ))
    }

    fn validate_transfer(&self, transfer: &TransferRequest) -> Result<(), CommunexError> {
        // Validate addresses
        if !transfer.from.starts_with("cmx1") {
            return Err(CommunexError::ValidationError(
                format!("Invalid sender address format: {}", transfer.from)
            ));
        }
        if !transfer.to.starts_with("cmx1") {
            return Err(CommunexError::ValidationError(
                format!("Invalid receiver address format: {}", transfer.to)
            ));
        }

        // Validate amount
        if transfer.amount < MIN_AMOUNT {
            return Err(CommunexError::ValidationError(
                format!("Amount must be greater than {}", MIN_AMOUNT - 1)
            ));
        }

        // Validate denomination
        if !VALID_DENOMS.contains(&transfer.denom.as_str()) {
            return Err(CommunexError::ValidationError(
                format!("Invalid denomination: {}. Valid options are: {:?}", 
                    transfer.denom, VALID_DENOMS)
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_request_new() {
        let request = TransferRequest {
            from: "cmx1abcd123".into(),
            to: "cmx1efgh456".into(),
            amount: 1000,
            denom: "COMAI".into(),
        };
        
        assert_eq!(request.from, "cmx1abcd123");
        assert_eq!(request.amount, 1000);
        assert_eq!(request.denom, "COMAI");
    }
}
use crate::{CommunexError, rpc::RpcClient};
use serde::{Serialize, Deserialize};
use serde_json::json;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub denom: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResponse {
    pub status: String,
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
    pub status: TransactionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Success,
    Failed,
    Pending,
}

pub struct WalletClient {
    pub rpc_client: RpcClient,
}

impl WalletClient {
    pub fn new(uri: &str) -> Self {
        WalletClient {
            rpc_client: RpcClient::new(uri),
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
                    status: response.get("status")
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
                            status: match tx.get("status").and_then(|v| v.as_str()) {
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
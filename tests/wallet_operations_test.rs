use comx_api::{
    wallet::{WalletClient, TransferRequest, Txstate, TransactionStatus, staking::StakeRequest},
    error::CommunexError,
};
use wiremock::{
    Mock, 
    MockServer,
    ResponseTemplate,
    matchers::{method, path, body_json}
};
use serde_json::json;

#[tokio::test]
async fn test_transfer_success() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/transfer"))
        .and(body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "transfer",
            "params": {
                "from": "cmx1abcd123",
                "to": "cmx1efgh456",
                "amount": "1000",
                "denom": "COMAI"
            }
        })))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "status": "success"
                }
            })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    
    let request = TransferRequest {
        from: "cmx1abcd123".into(),
        to: "cmx1efgh456".into(),
        amount: 1000,
        denom: "COMAI".into(),
    };
    
    let result = client.transfer(request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_transfer_insufficient_funds() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/transfer"))
        .and(body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "transfer",
            "params": {
                "from": "cmx1abcd123",
                "to": "cmx1efgh456",
                "amount": "1000000000",
                "denom": "COMAI"
            }
        })))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "error": {
                    "code": -32000,
                    "message": "insufficient funds"
                }
            })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    
    let request = TransferRequest {
        from: "cmx1abcd123".into(),
        to: "cmx1efgh456".into(),
        amount: 1000000000,
        denom: "COMAI".into(),
    };
    
    let result = client.transfer(request).await;
    assert!(matches!(
        result,
        Err(CommunexError::RpcError { code: -32000, .. })
    ));
}

#[tokio::test]
async fn test_get_free_balance() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/balance/free"))
        .and(body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "balance/free",
            "params": {
                "address": "cmx1abcd123"
            }
        })))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "free": 1000000
                }
            })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    let balance = client.get_free_balance("cmx1abcd123").await.unwrap();
    assert_eq!(balance, 1000000);
}

#[tokio::test]
async fn test_get_all_balances() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/balance/all"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "free": 1000000,
                    "reserved": 50000,
                    "miscFrozen": 10000,
                    "feeFrozen": 5000
                }
            })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    let balances = client.get_all_balances("cmx1abcd123").await.unwrap();
    
    assert_eq!(balances.free, 1000000);
    assert_eq!(balances.reserved, 50000);
    assert_eq!(balances.misc_frozen, 10000);
    assert_eq!(balances.fee_frozen, 5000);
}

#[tokio::test]
async fn test_get_transaction_history() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/transaction/history"))
        .and(body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "transaction/history",
            "params": {
                "address": "cmx1abcd123"
            }
        })))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "transactions": [
                        {
                            "hash": "0x123...",
                            "block_num": 12345,
                            "timestamp": 1704067200,
                            "from": "cmx1sender",
                            "to": "cmx1receiver",
                            "amount": 1000,
                            "denom": "COMAI",
                            "state": "success"
                        },
                        {
                            "hash": "0x456...",
                            "block_num": 12346,
                            "timestamp": 1704067260,
                            "from": "cmx1sender",
                            "to": "cmx1receiver",
                            "amount": 2000,
                            "denom": "COMAI",
                            "state": "pending"
                        }
                    ]
                }
            })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    let history = client.get_transaction_history("cmx1abcd123").await.unwrap();
    
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].hash, "0x123...");
    assert_eq!(history[0].amount, 1000);
    assert!(matches!(history[0].state, TransactionStatus::Success));
    assert_eq!(history[1].hash, "0x456...");
    assert_eq!(history[1].amount, 2000);
    assert!(matches!(history[1].state, TransactionStatus::Pending));
}

#[tokio::test]
async fn test_get_transaction_history_invalid_address() {
    let mock_server = MockServer::start().await;
    let client = WalletClient::new(&mock_server.uri());
    
    let result = client.get_transaction_history("invalid_address").await;
    assert!(matches!(
        result,
        Err(CommunexError::RpcError { code: -32001, .. })
    ));
}

#[tokio::test]
async fn test_stake_tokens() -> Result<(), CommunexError> {
    let mock_server = MockServer::start().await;
    
    // Mock the stake request with the correct path and body matcher
    Mock::given(method("POST"))
        .and(path("/staking/stake"))
        .and(body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "staking/stake",
            "params": {
                "from": "cmx1sender...",
                "amount": 1000,
                "denom": "COMAI"
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "hash": "0xabcd1234",
                "state": "success"
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock the transaction status check
    Mock::given(method("POST"))
        .and(path("/transaction/state"))
        .and(body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "transaction/state",
            "params": {
                "hash": "0xabcd1234"
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "state": "success",
                "hash": "0xabcd1234",
                "confirmations": 1,
                "block_num": 12345,
                "timestamp": 1704067200,
                "error": null
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    let stake_request = StakeRequest {
        from: "cmx1sender...".to_string(),
        amount: 1000,
        denom: "COMAI".to_string(),
    };

    let result = client.stake(stake_request).await?;
    assert!(matches!(result.state, Txstate::Success));

    Ok(())
}

#[tokio::test]
async fn test_get_staking_info() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/staking/info"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "total_staked": 5000,
                    "rewards_available": 100,
                    "last_claim_time": 1704067200,
                    "denom": "COMAI"
                }
            })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    let info = client.get_staking_info("cmx1abcd123").await.unwrap();
    
    assert_eq!(info.total_staked, 5000);
    assert_eq!(info.rewards_available, 100);
    assert_eq!(info.denom, "COMAI");
}

#[tokio::test]
async fn test_get_transaction_status() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/transaction/state"))
        .and(body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "transaction/state",
            "params": {
                "hash": "0x123..."
            }
        })))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "block_num": 12345,
                    "confirmations": 5,
                    "state": "success",
                    "timestamp": 1704067200,
                    "error": null
                }
            })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    let status = client.get_transaction_state("0x123...").await.unwrap();
    
    assert_eq!(status.hash, "0x123...");
    assert_eq!(status.block_num, Some(12345));
    assert_eq!(status.confirmations, 5);
    assert!(matches!(status.state, Txstate::Success));
    assert!(status.error.is_none());
} 
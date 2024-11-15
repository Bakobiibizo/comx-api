use comx_api::{
    wallet::{WalletClient, TransferRequest, TransactionStatus},
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
async fn test_batch_transfer_success() {
    let mock_server = MockServer::start().await;
    
    // Setup mock response for batch transfer
    Mock::given(method("POST"))
        .and(path("/batch_transfer"))
        .and(body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "batch_transfer",
            "params": {
                "transfers": [
                    {
                        "from": "cmx1sender",
                        "to": "cmx1receiver1",
                        "amount": 100,
                        "denom": "COMAI"
                    },
                    {
                        "from": "cmx1sender",
                        "to": "cmx1receiver2",
                        "amount": 200,
                        "denom": "COMAI"
                    }
                ]
            }
        })))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "batch_id": "batch123",
                    "transactions": [
                        {"hash": "tx1hash", "status": "pending"},
                        {"hash": "tx2hash", "status": "pending"}
                    ]
                }
            })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    
    let transfers = vec![
        TransferRequest {
            from: "cmx1sender".into(),
            to: "cmx1receiver1".into(),
            amount: 100,
            denom: "COMAI".into(),
        },
        TransferRequest {
            from: "cmx1sender".into(),
            to: "cmx1receiver2".into(),
            amount: 200,
            denom: "COMAI".into(),
        },
    ];

    let result = client.batch_transfer(transfers).await;
    assert!(result.is_ok());
    
    let batch_result = result.unwrap();
    assert_eq!(batch_result.batch_id, "batch123");
    assert_eq!(batch_result.transactions.len(), 2);
}

#[tokio::test]
async fn test_batch_transfer_partial_failure() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/batch_transfer"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "batch_id": "batch123",
                    "transactions": [
                        {"hash": "tx1hash", "status": "success"},
                        {"hash": "tx2hash", "status": "failed", "error": "insufficient funds"}
                    ]
                }
            })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    
    let transfers = vec![
        TransferRequest {
            from: "cmx1sender".into(),
            to: "cmx1receiver1".into(),
            amount: 100,
            denom: "COMAI".into(),
        },
        TransferRequest {
            from: "cmx1sender".into(),
            to: "cmx1receiver2".into(),
            amount: 999999,  // Amount too high
            denom: "COMAI".into(),
        },
    ];

    let result = client.batch_transfer(transfers).await;
    assert!(result.is_ok());
    
    let batch_result = result.unwrap();
    assert_eq!(batch_result.transactions[0].status, TransactionStatus::Success);
    assert_eq!(batch_result.transactions[1].status, TransactionStatus::Failed);
}

#[tokio::test]
async fn test_batch_transfer_empty_list() {
    let mock_server = MockServer::start().await;
    let client = WalletClient::new(&mock_server.uri());
    
    let result = client.batch_transfer(vec![]).await;
    assert!(matches!(result, Err(CommunexError::ValidationError(_))));
}

#[tokio::test]
async fn test_batch_transfer_too_many_requests() {
    let mock_server = MockServer::start().await;
    let client = WalletClient::new(&mock_server.uri());
    
    // Create 101 transfer requests (assuming max batch size is 100)
    let transfers = (0..101).map(|i| TransferRequest {
        from: "cmx1sender".into(),
        to: format!("cmx1receiver{}", i),
        amount: 100,
        denom: "COMAI".into(),
    }).collect();

    let result = client.batch_transfer(transfers).await;
    assert!(matches!(result, Err(CommunexError::ValidationError(_))));
}

#[tokio::test]
async fn test_batch_transfer_invalid_addresses() {
    let mock_server = MockServer::start().await;
    let client = WalletClient::new(&mock_server.uri());
    
    let transfers = vec![
        TransferRequest {
            from: "invalid_sender".into(),  // Invalid sender address
            to: "cmx1receiver1".into(),
            amount: 100,
            denom: "COMAI".into(),
        },
        TransferRequest {
            from: "cmx1sender".into(),
            to: "invalid_receiver".into(),  // Invalid receiver address
            amount: 200,
            denom: "COMAI".into(),
        },
    ];

    let result = client.batch_transfer(transfers).await;
    assert!(matches!(result, Err(CommunexError::ValidationError(_))));
}

#[tokio::test]
async fn test_batch_transfer_invalid_amounts() {
    let mock_server = MockServer::start().await;
    let client = WalletClient::new(&mock_server.uri());
    
    let transfers = vec![
        TransferRequest {
            from: "cmx1sender".into(),
            to: "cmx1receiver1".into(),
            amount: 0,  // Invalid amount
            denom: "COMAI".into(),
        },
    ];

    let result = client.batch_transfer(transfers).await;
    assert!(matches!(result, Err(CommunexError::ValidationError(_))));
}

#[tokio::test]
async fn test_batch_transfer_invalid_denom() {
    let mock_server = MockServer::start().await;
    let client = WalletClient::new(&mock_server.uri());
    
    let transfers = vec![
        TransferRequest {
            from: "cmx1sender".into(),
            to: "cmx1receiver1".into(),
            amount: 100,
            denom: "INVALID".into(),  // Invalid denomination
        },
    ];

    let result = client.batch_transfer(transfers).await;
    assert!(matches!(result, Err(CommunexError::ValidationError(_))));
}

#[tokio::test]
async fn test_batch_transfer_server_error() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/batch_transfer"))
        .respond_with(ResponseTemplate::new(500)
            .set_body_json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "error": {
                    "code": -32603,
                    "message": "Internal server error"
                }
            })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    
    let transfers = vec![
        TransferRequest {
            from: "cmx1sender".into(),
            to: "cmx1receiver1".into(),
            amount: 100,
            denom: "COMAI".into(),
        },
    ];

    let result = client.batch_transfer(transfers).await;
    assert!(matches!(result, Err(CommunexError::RpcError { .. })));
}

#[tokio::test]
async fn test_batch_transfer_timeout() {
    let mock_server = MockServer::start().await;
    
    // Configure mock to delay response beyond client timeout
    Mock::given(method("POST"))
        .and(path("/batch_transfer"))
        .respond_with(ResponseTemplate::new(200)
            .set_delay(std::time::Duration::from_secs(5)))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    
    let transfers = vec![
        TransferRequest {
            from: "cmx1sender".into(),
            to: "cmx1receiver1".into(),
            amount: 100,
            denom: "COMAI".into(),
        },
    ];

    let result = client.batch_transfer(transfers).await;
    assert!(matches!(result, Err(CommunexError::RequestTimeout(_))));
}

#[tokio::test]
async fn test_batch_transfer_malformed_response() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/batch_transfer"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "batch_id": "batch123",
                    "transactions": [
                        {"invalid_field": "value"}  // Malformed transaction status
                    ]
                }
            })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    
    let transfers = vec![
        TransferRequest {
            from: "cmx1sender".into(),
            to: "cmx1receiver1".into(),
            amount: 100,
            denom: "COMAI".into(),
        },
    ];

    let result = client.batch_transfer(transfers).await;
    assert!(matches!(result, Err(CommunexError::ParseError(_))));
}

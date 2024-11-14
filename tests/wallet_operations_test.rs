use comx_api::{
    wallet::{WalletClient, TransferRequest, TransferResponse},
    error::CommunexError,
};
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate
};
use std::time::Duration;

#[tokio::test]
async fn test_transfer_success() -> Result<(), CommunexError> {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/transfer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"status": "success"}
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    let request = TransferRequest {
        from: "cmx1sender...".to_string(),
        to: "cmx1receiver...".to_string(),
        amount: 1000000,
        denom: "COMAI".to_string(),
    };
    let response: TransferResponse = client.transfer(request).await?;
    
    assert_eq!(response.status, "success");
    Ok(())
}

#[tokio::test]
async fn test_transfer_insufficient_funds() -> Result<(), CommunexError> {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/transfer"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {"code": -32000, "message": "Insufficient funds"}
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    let request = TransferRequest {
        from: "cmx1sender...".to_string(),
        to: "cmx1receiver...".to_string(),
        amount: 1000000000, // Exceeds balance
        denom: "COMAI".to_string(),
    };
    let result = client.transfer(request).await;
    
    assert!(matches!(result, Err(CommunexError::RpcError { code: -32000, .. })));
    Ok(())
}

#[tokio::test]
async fn test_transfer_invalid_address() -> Result<(), CommunexError> {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/transfer"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {"code": -32001, "message": "Invalid address"}
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    let request = TransferRequest {
        from: "invalid_address".to_string(),
        to: "cmx1receiver...".to_string(),
        amount: 1000,
        denom: "COMAI".to_string(),
    };
    let result = client.transfer(request).await;
    
    assert!(matches!(result, Err(CommunexError::RpcError { code: -32001, .. })));
    Ok(())
}

#[tokio::test]
async fn test_transfer_zero_amount() -> Result<(), CommunexError> {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/transfer"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {"code": -32002, "message": "Amount must be greater than zero"}
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    let request = TransferRequest {
        from: "cmx1sender...".to_string(),
        to: "cmx1receiver...".to_string(),
        amount: 0,
        denom: "COMAI".to_string(),
    };
    let result = client.transfer(request).await;
    
    assert!(matches!(result, Err(CommunexError::RpcError { code: -32002, .. })));
    Ok(())
}

#[tokio::test]
async fn test_transfer_network_error() -> Result<(), CommunexError> {
    let client = WalletClient::new(&"http://invalid-server");
    let request = TransferRequest {
        from: "cmx1sender...".to_string(),
        to: "cmx1receiver...".to_string(),
        amount: 1000,
        denom: "COMAI".to_string(),
    };
    let result = client.transfer(request).await;
    
    assert!(matches!(result, Err(CommunexError::ConnectionError(_))));
    Ok(())
}

#[tokio::test]
async fn test_transfer_large_amount() -> Result<(), CommunexError> {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/transfer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"status": "success"}
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    let request = TransferRequest {
        from: "cmx1sender...".to_string(),
        to: "cmx1receiver...".to_string(),
        amount: u64::MAX,
        denom: "COMAI".to_string(),
    };
    let response: TransferResponse = client.transfer(request).await?;
    
    assert_eq!(response.status, "success");
    Ok(())
}

#[tokio::test]
async fn test_transfer_unsupported_denom() -> Result<(), CommunexError> {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/transfer"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {"code": -32003, "message": "Unsupported denomination"}
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = WalletClient::new(&mock_server.uri());
    let request = TransferRequest {
        from: "cmx1sender...".to_string(),
        to: "cmx1receiver...".to_string(),
        amount: 1000,
        denom: "UNSUPPORTED".to_string(),
    };
    let result = client.transfer(request).await;
    
    assert!(matches!(result, Err(CommunexError::RpcError { code: -32003, .. })));
    Ok(())
} 
use comx_api::{
    wallet::{WalletClient, TransferRequest},
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
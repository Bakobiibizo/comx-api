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
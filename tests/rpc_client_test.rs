use comx_api::rpc::{RpcClient, BatchRequest};
use comx_api::types::{Balance, Address, FromRpcResponse};
use comx_api::error::CommunexError;
use serde_json::{json, Value};
use tokio;
use std::time::Duration;
use mockito::{Server, Mock};

#[tokio::test]

async fn test_single_rpc_request() {
    let mut server = Server::new();
    let mock_response = json!({
        "jsonrpc": "2.0",
        "result": {
            "amount": "1000000",
            "denom": "COMAI"
        },
        "id": 1
    });

    // Setup mock server
    let _m = server.mock("POST", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let client = RpcClient::new(server.url());
    
    let response = client.request(
        "query_balance",
        json!({
            "address": "cmx1abc123def456"
        })
    ).await;

    assert!(response.is_ok());
    let balance = Balance::from_rpc(response.unwrap()).unwrap();
    assert_eq!(balance.amount(), 1000000);
    assert_eq!(balance.denom(), "COMAI");
}

#[tokio::test]
async fn test_batch_request() {
    let mut server = Server::new();
    let mock_response = json!([
        {
            "jsonrpc": "2.0",
            "result": {
                "amount": "1000000",
                "denom": "COMAI"
            },
            "id": 1
        },
        {
            "jsonrpc": "2.0",
            "result": {
                "amount": "2000000",
                "denom": "COMAI"
            },
            "id": 2
        }
    ]);

    // Setup mock server
    let _m = server.mock("POST", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let client = RpcClient::new(server.url());
    let mut batch = BatchRequest::new();

    batch.add_request("query_balance", json!({"address": "cmx1abc123"}));
    batch.add_request("query_balance", json!({"address": "cmx1def456"}));

    let responses = client.batch_request(batch).await;
    assert!(responses.is_ok());
    
    let responses = responses.unwrap();
    assert_eq!(responses.len(), 2);
    
    let first_balance = Balance::from_rpc(responses[0].clone()).unwrap();
    let second_balance = Balance::from_rpc(responses[1].clone()).unwrap();
    
    assert_eq!(first_balance.amount(), 1000000);
    assert_eq!(second_balance.amount(), 2000000);
}

#[tokio::test]
async fn test_rpc_error_handling() {
    let mut server = Server::new();
    let mock_response = json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32601,
            "message": "Method not found"
        },
        "id": 1
    });

    // Setup mock server
    let _m = server.mock("POST", "/")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let client = RpcClient::new(server.url());
    let response = client.request(
        "invalid_method",
        json!({})
    ).await;

    assert!(response.is_err());
    assert!(matches!(response.unwrap_err(), CommunexError::RpcError { .. }));
}

#[tokio::test]
async fn test_connection_timeout() {
    let client = RpcClient::with_timeout(
        "http://non.existent.host",
        Duration::from_millis(100)
    );

    let response = client.request(
        "query_balance",
        json!({
            "address": "cmx1abc123def456"
        })
    ).await;

    assert!(response.is_err());
    assert!(matches!(response.unwrap_err(), CommunexError::ConnectionError { .. }));
} 
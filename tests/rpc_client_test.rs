use comx_api::rpc::{RpcClient, BatchRequest};
use comx_api::types::{Balance, FromRpcResponse};
use comx_api::error::CommunexError;
use serde_json::json;
use std::time::Duration;
use mockito::{Server, ServerOpts};
use serial_test::serial;

async fn setup_test_server(response: serde_json::Value) -> (Server, RpcClient) {
    let opts = ServerOpts::default();
    let mut server = Server::new_with_opts_async(opts).await;
    
    let _m = server.mock("POST", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response.to_string())
        .create();

    let client = RpcClient::new(server.url());
    (server, client)
}

#[tokio::test]
#[serial]
async fn test_single_rpc_request() {
    let mock_response = json!({
        "jsonrpc": "2.0",
        "result": {
            "amount": "1000000",
            "denom": "COMAI"
        },
        "id": 1
    });

    let (_server, client) = setup_test_server(mock_response).await;
    
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
#[serial]
async fn test_batch_request() {
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

    let (_server, client) = setup_test_server(mock_response).await;
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
#[serial]
async fn test_rpc_error_handling() {
    let mock_response = json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32601,
            "message": "Method not found"
        },
        "id": 1
    });

    let (_server, client) = setup_test_server(mock_response).await;
    
    let response = client.request(
        "invalid_method",
        json!({})
    ).await;

    assert!(response.is_err());
    assert!(matches!(response.unwrap_err(), CommunexError::RpcError { .. }));
}

#[tokio::test]
#[serial]
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
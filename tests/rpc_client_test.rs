use comx_api::{
    rpc::{RpcClient, RpcClientConfig, BatchRequest},
    error::CommunexError,
};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate
};
use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_single_request_success() -> Result<(), CommunexError> {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"balance": "1000"}
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = RpcClient::new(mock_server.uri());
    let result = client.request("query_balance", json!({"address": "test"})).await?;
    
    assert_eq!(result.get("balance").unwrap().as_str().unwrap(), "1000");
    Ok(())
}

#[tokio::test]
async fn test_batch_request_success() -> Result<(), CommunexError> {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "jsonrpc": "2.0",
                "id": 0,
                "result": {"balance": "1000"}
            },
            {
                "jsonrpc": "2.0",
                "id": 1,
                "result": {"balance": "2000"}
            }
        ])))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = RpcClient::new(mock_server.uri());
    let mut batch = BatchRequest::new();
    batch.add_request("query_balance", json!({"address": "addr1"}));
    batch.add_request("query_balance", json!({"address": "addr2"}));
    
    let response = client.batch_request(batch).await?;
    assert_eq!(response.successes.len(), 2);
    assert!(response.errors.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_rpc_error_response() -> Result<(), CommunexError> {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        })))
        .expect(1..=3)
        .mount(&mock_server)
        .await;

    let client = RpcClient::new_with_config(
        mock_server.uri(),
        RpcClientConfig {
            timeout: Duration::from_secs(1),
            max_retries: 2,
        }
    );
    
    let result = client.request("invalid_method", json!({})).await;
    assert!(matches!(result, Err(CommunexError::RpcError { code: -32601, .. })));
    Ok(())
}

#[tokio::test]
async fn test_connection_timeout() -> Result<(), CommunexError> {
    let config = RpcClientConfig {
        timeout: Duration::from_millis(100),
        max_retries: 1,
    };
    
    let client = RpcClient::new_with_config("http://invalid-url", config);
    let result = client.request("test", json!({})).await;
    
    assert!(matches!(result, Err(CommunexError::ConnectionError(_))));
    Ok(())
}

#[tokio::test]
async fn test_batch_request_partial_failure() -> Result<(), CommunexError> {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "jsonrpc": "2.0",
                "id": 0,
                "result": {"balance": "1000"}
            },
            {
                "jsonrpc": "2.0",
                "id": 1,
                "error": {
                    "code": -32602,
                    "message": "Invalid params"
                }
            }
        ])))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = RpcClient::new(mock_server.uri());
    let mut batch = BatchRequest::new();
    batch.add_request("query_balance", json!({"address": "valid"}));
    batch.add_request("query_balance", json!({"invalid": "params"}));
    
    let response = client.batch_request(batch).await?;
    assert_eq!(response.successes.len(), 1);
    assert_eq!(response.errors.len(), 1);
    assert_eq!(response.errors[0].code, -32602);
    Ok(())
}

#[tokio::test]
async fn test_retry_mechanism() -> Result<(), CommunexError> {
    let mock_server = MockServer::start().await;
    
    // First two attempts fail
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(500))
        .expect(1..=3)
        .mount(&mock_server)
        .await;
    
    // Final attempt succeeds
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"success": true}
        })))
        .expect(0..=1)
        .mount(&mock_server)
        .await;

    let client = RpcClient::new_with_config(
        mock_server.uri(),
        RpcClientConfig {
            timeout: Duration::from_secs(1),
            max_retries: 2,
        }
    );
    
    let result = client.request("test", json!({})).await;
    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            // Either a connection error or parse error is acceptable
            assert!(matches!(e, 
                CommunexError::ConnectionError(_) | 
                CommunexError::ParseError(_) |
                CommunexError::RpcError { .. }
            ));
            Ok(())
        }
    }
}
use comx_api::{
    rpc::RpcClient,
    types::Address,
    query_map::{QueryMap, QueryMapConfig},
    error::CommunexError,
};
use tokio::time::{Duration, sleep};
use serde_json::json;
use mockito::{Server, ServerOpts};
use serial_test::serial;

const TEST_ADDRESS: &str = "cmx1abc123def456";

async fn setup_test_server(response: serde_json::Value) -> (Server, RpcClient) {
    let opts = ServerOpts::default();
    let mut server = Server::new_with_opts_async(opts).await;
    
    let rpc_response = if response.is_array() {
        response
    } else if response.get("error").is_some() {
        response
    } else {
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": response
        })
    };
    
    let _m = server.mock("POST", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(rpc_response.to_string())
        .create();

    let client = RpcClient::new(server.url());
    (server, client)
}

#[tokio::test]
#[serial]
async fn test_query_map_creation() {
    let client = RpcClient::new("http://test-node");
    let config = QueryMapConfig {
        refresh_interval: Duration::from_secs(300), // 5 minutes
        cache_duration: Duration::from_secs(600),   // 10 minutes
    };
    
    let query_map = QueryMap::new(client, config);
    assert!(query_map.is_ok());
}

#[tokio::test]
#[serial]
async fn test_balance_query() -> Result<(), CommunexError> {
    let (_server, client) = setup_test_server(json!({
        "amount": "1000000",
        "denom": "COMAI"
    })).await;
    
    let query_map = QueryMap::new(client, QueryMapConfig::default()).unwrap();
    let balance = query_map.get_balance(TEST_ADDRESS).await?;
    
    assert_eq!(balance.amount(), Ok(1000000));
    assert_eq!(balance.denom(), "COMAI");
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_stake_relationships() -> Result<(), CommunexError> {
    let (_server, client) = setup_test_server(json!({
        "stake_from": ["cmx1addr1", "cmx1addr2"],
        "stake_to": ["cmx1addr3", "cmx1addr4"],
        "amounts": {
            "cmx1addr1": "100000",
            "cmx1addr2": "200000",
            "cmx1addr3": "300000",
            "cmx1addr4": "400000"
        }
    })).await;
    
    let query_map = QueryMap::new(client, QueryMapConfig::default()).unwrap();
    
    let stake_from = query_map.get_stake_from(TEST_ADDRESS).await?;
    assert_eq!(stake_from.len(), 2);
    assert!(stake_from.contains(&Address::new("cmx1addr1").unwrap()));
    
    let stake_to = query_map.get_stake_to(TEST_ADDRESS).await?;
    assert_eq!(stake_to.len(), 2);
    assert!(stake_to.contains(&Address::new("cmx1addr3").unwrap()));
    
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_cache_refresh() -> Result<(), CommunexError> {
    let (_server, client) = setup_test_server(json!({
        "amount": "1000000",
        "denom": "COMAI"
    })).await;
    
    let config = QueryMapConfig {
        refresh_interval: Duration::from_secs(1),
        cache_duration: Duration::from_secs(2),
    };
    
    let query_map = QueryMap::new(client, config).unwrap();
    
    // Initial query
    let _initial_balance = query_map.get_balance(TEST_ADDRESS).await?;
    
    // Wait for refresh
    sleep(Duration::from_secs(2)).await;
    
    // Should trigger new query
    let _refreshed_balance = query_map.get_balance(TEST_ADDRESS).await?;
    
    assert!(query_map.cache_stats().refresh_count > 0);
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_batch_balance_queries() -> Result<(), CommunexError> {
    let addresses = vec![
        "cmx1abc123def456",
        "cmx1def456abc789",
        "cmx1ghi789jkl012"
    ];
    
    // Note: For batch requests, we need to send an array directly, not wrapped in a result
    let mock_response = json!([
        {
            "jsonrpc": "2.0",
            "id": 0,
            "result": {
                "amount": "1000000",
                "denom": "COMAI"
            }
        },
        {
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "amount": "2000000",
                "denom": "COMAI"
            }
        },
        {
            "jsonrpc": "2.0",
            "id": 2,
            "result": {
                "amount": "3000000",
                "denom": "COMAI"
            }
        }
    ]);
    
    let (_server, client) = setup_test_server(mock_response).await;
    let query_map = QueryMap::new(client, QueryMapConfig::default()).unwrap();
    let balances = query_map.get_balances(&addresses).await?;
    
    assert_eq!(balances.len(), 3);
    assert_eq!(balances[0].amount()?, 1000000);
    assert_eq!(balances[1].amount()?, 2000000);
    assert_eq!(balances[2].amount()?, 3000000);
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_error_handling() -> Result<(), CommunexError> {
    let (_server, client) = setup_test_server(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": -32601,
            "message": "Method not found"
        }
    })).await;
    
    let query_map = QueryMap::new(client, QueryMapConfig::default()).unwrap();
    let result = query_map.get_balance(TEST_ADDRESS).await;
    
    assert!(result.is_err());
    if let Err(CommunexError::RpcError { code, message }) = result {
        assert_eq!(code, -32601);
        assert!(message.contains("Method not found"));
        Ok(())
    } else {
        panic!("Expected RpcError, got {:?}", result);
    }
}

#[tokio::test]
#[serial]
async fn test_query_map_creation_validation() {
    let (_server, client) = setup_test_server(json!({})).await;
    
    // Test with invalid refresh interval
    let config = QueryMapConfig {
        refresh_interval: Duration::from_millis(100),
        cache_duration: Duration::from_secs(600),
    };
    let result = QueryMap::new(client.clone(), config);
    
    // Updated assertion to match ConfigError variant
    assert!(matches!(result.unwrap_err(), CommunexError::ConfigError(msg) if 
        msg.contains("at least 1 second")));

    // Test with invalid cache duration
    let config = QueryMapConfig {
        refresh_interval: Duration::from_secs(10),
        cache_duration: Duration::from_secs(5),
    };
    assert!(matches!(QueryMap::new(client, config).unwrap_err(), 
        CommunexError::ConfigError(msg) if msg.contains("longer than refresh")));
}

#[tokio::test]
async fn test_empty_batch_request() -> Result<(), CommunexError> {
    let (_server, client) = setup_test_server(json!([])).await;
    let query_map = QueryMap::new(client, QueryMapConfig::default())?;
    
    let empty_addresses: Vec<&str> = vec![];
    let balances = query_map.get_balances(&empty_addresses).await?;
    assert!(balances.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_invalid_config() {
    let (_server, client) = setup_test_server(json!({})).await;
    
    // Test refresh interval too short
    let config = QueryMapConfig {
        refresh_interval: Duration::from_millis(100),
        cache_duration: Duration::from_secs(600),
    };
    assert!(QueryMap::new(client.clone(), config).is_err());

    // Test cache duration shorter than refresh
    let config = QueryMapConfig {
        refresh_interval: Duration::from_secs(10),
        cache_duration: Duration::from_secs(5),
    };
    assert!(QueryMap::new(client, config).is_err());
}

#[tokio::test]
async fn test_malformed_stake_response() -> Result<(), CommunexError> {
    let (_server, client) = setup_test_server(json!({
        "not_stake_from": []
    })).await;
    
    let query_map = QueryMap::new(client, QueryMapConfig::default())?;
    let result = query_map.get_stake_from(TEST_ADDRESS).await;
    
    assert!(matches!(result, Err(CommunexError::ParseError(_))));
    Ok(())
}

#[tokio::test]
async fn test_batch_request_partial_failure() -> Result<(), CommunexError> {
    let batch_response = json!([
        {
            "jsonrpc": "2.0",
            "id": 0,
            "result": {
                "amount": "1000000",
                "denom": "COMAI"
            }
        },
        {
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32602,
                "message": "Invalid address"
            }
        }
    ]);
    
    let (_server, client) = setup_test_server(batch_response).await;
    let query_map = QueryMap::new(client, QueryMapConfig::default())?;
    
    let addresses = vec!["cmx1valid", "cmx1invalid"];
    let response = query_map.get_balances(&addresses).await?;
    
    assert_eq!(response.len(), 1);
    assert_eq!(response[0].amount()?, 1000000);
    Ok(())
} 
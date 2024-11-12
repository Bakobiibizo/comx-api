use comx_api::rpc::RpcClient;
use comx_api::query_map::{QueryMap, QueryMapConfig};
use comx_api::types::{Address, Balance};
use comx_api::error::CommunexError;
use tokio::time::{Duration, sleep};
use std::sync::Arc;
use serial_test::serial;

const TEST_ADDRESS: &str = "cmx1abc123def456";
const TEST_STAKE_ADDRESS: &str = "cmx1def456abc789";

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
async fn test_balance_query() {
    let (_server, client) = setup_test_server(json!({
        "amount": "1000000",
        "denom": "COMAI"
    })).await;
    
    let query_map = QueryMap::new(client, QueryMapConfig::default()).unwrap();
    let balance = query_map.get_balance(TEST_ADDRESS).await;
    
    assert!(balance.is_ok());
    let balance = balance.unwrap();
    assert_eq!(balance.amount(), "1000000");
    assert_eq!(balance.denom(), "COMAI");
}

#[tokio::test]
#[serial]
async fn test_stake_relationships() {
    let (_server, client) = setup_test_server(json!({
        "stake_from": ["addr1", "addr2"],
        "stake_to": ["addr3", "addr4"],
        "amounts": {
            "addr1": "100000",
            "addr2": "200000",
            "addr3": "300000",
            "addr4": "400000"
        }
    })).await;
    
    let query_map = QueryMap::new(client, QueryMapConfig::default()).unwrap();
    
    let stake_from = query_map.get_stake_from(TEST_ADDRESS).await?;
    assert_eq!(stake_from.len(), 2);
    assert!(stake_from.contains(&Address::new("addr1").unwrap()));
    
    let stake_to = query_map.get_stake_to(TEST_ADDRESS).await?;
    assert_eq!(stake_to.len(), 2);
    assert!(stake_to.contains(&Address::new("addr3").unwrap()));
}

#[tokio::test]
#[serial]
async fn test_cache_refresh() {
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
    let balance1 = query_map.get_balance(TEST_ADDRESS).await?;
    
    // Wait for refresh
    sleep(Duration::from_secs(2)).await;
    
    // Should trigger new query
    let balance2 = query_map.get_balance(TEST_ADDRESS).await?;
    
    assert!(query_map.cache_stats().refresh_count > 0);
}

#[tokio::test]
#[serial]
async fn test_batch_balance_queries() {
    let addresses = vec![
        "cmx1abc123def456",
        "cmx1def456abc789",
        "cmx1ghi789jkl012"
    ];
    
    let (_server, client) = setup_test_server(json!({
        "balances": {
            "cmx1abc123def456": {"amount": "1000000", "denom": "COMAI"},
            "cmx1def456abc789": {"amount": "2000000", "denom": "COMAI"},
            "cmx1ghi789jkl012": {"amount": "3000000", "denom": "COMAI"}
        }
    })).await;
    
    let query_map = QueryMap::new(client, QueryMapConfig::default()).unwrap();
    let balances = query_map.get_balances(&addresses).await?;
    
    assert_eq!(balances.len(), 3);
    assert_eq!(balances[0].amount(), "1000000");
    assert_eq!(balances[1].amount(), "2000000");
    assert_eq!(balances[2].amount(), "3000000");
}

#[tokio::test]
#[serial]
async fn test_error_handling() {
    let (_server, client) = setup_test_server(json!({
        "error": {
            "code": -32601,
            "message": "Method not found"
        }
    })).await;
    
    let query_map = QueryMap::new(client, QueryMapConfig::default()).unwrap();
    let result = query_map.get_balance(TEST_ADDRESS).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CommunexError::RpcError { .. }));
} 
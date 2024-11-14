use tokio::time::sleep;
use std::time::Duration;
use std::sync::Arc;
use crate::cache::{QueryMapCache, CacheConfig, QueryResult};

#[tokio::test]
async fn test_cache_basic_operations() {
    let config = CacheConfig {
        ttl: Duration::from_secs(60),
        refresh_interval: Duration::from_secs(300),
        max_entries: 1000,
    };
    
    let cache = QueryMapCache::new(config);
    
    let query_key = "test_query";
    let test_data = QueryResult::new("test_value");
    
    cache.set(query_key, test_data.clone()).await;
    let cached_data = cache.get(query_key).await;
    
    assert!(cached_data.is_some());
    assert_eq!(cached_data.unwrap(), test_data);
}

#[tokio::test]
async fn test_cache_ttl_expiration() {
    let config = CacheConfig {
        ttl: Duration::from_secs(1),
        refresh_interval: Duration::from_secs(300),
        max_entries: 1000,
    };
    
    let cache = QueryMapCache::new(config);
    let query_key = "expiring_query";
    let test_data = QueryResult::new("expiring_value");
    
    cache.set(query_key, test_data).await;
    
    // Wait for TTL to expire
    sleep(Duration::from_secs(2)).await;
    
    let cached_data = cache.get(query_key).await;
    assert!(cached_data.is_none());
}

#[tokio::test]
async fn test_cache_memory_limits() {
    let config = CacheConfig {
        ttl: Duration::from_secs(60),
        refresh_interval: Duration::from_secs(300),
        max_entries: 5,
    };
    
    let cache = QueryMapCache::new(config);
    
    // Add more items than the cache limit
    for i in 0..10 {
        let key = format!("key_{}", i);
        let data = QueryResult::new(&format!("value_{}", i));
        cache.set(&key, data).await;
    }
    
    // Verify oldest entries were evicted
    assert!(cache.get("key_0").await.is_none());
    assert!(cache.get("key_9").await.is_some());
    
    let stats = cache.get_metrics().await;
    assert_eq!(stats.current_entries, 5);
}

#[tokio::test]
async fn test_cache_metrics() {
    let config = CacheConfig {
        ttl: Duration::from_secs(60),
        refresh_interval: Duration::from_secs(300),
        max_entries: 1000,
    };
    
    let cache = QueryMapCache::new(config);
    
    // Test hit
    let query_key = "metrics_test";
    let test_data = QueryResult::new("test_value");
    cache.set(query_key, test_data).await;
    let _ = cache.get(query_key).await;
    
    // Test miss
    let _ = cache.get("non_existent_key").await;
    
    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.hits, 1);
    assert_eq!(metrics.misses, 1);
}

#[tokio::test]
async fn test_background_refresh() {
    let config = CacheConfig {
        ttl: Duration::from_secs(1),
        refresh_interval: Duration::from_millis(100),
        max_entries: 1000,
    };
    
    let cache = Arc::new(QueryMapCache::new(config));
    
    // Setup refresh handler
    cache.set_refresh_handler(Box::new(|key: &str| {
        let key = key.to_string();
        Box::pin(async move {
            Ok(QueryResult::new(&format!("refreshed_{}", key)))
        })
    })).await;
    
    // Add initial data
    let query_key = "refresh_test";
    let initial_data = QueryResult::new("initial_value");
    cache.set(query_key, initial_data).await;
    
    // Force expire the entry
    cache.force_expire(query_key).await;
    
    // Start background refresh
    cache.start_background_refresh().await;
    
    // Wait for refresh cycle plus a small buffer
    sleep(Duration::from_millis(200)).await;
    
    // Try multiple times to get the refreshed data
    let mut attempts = 0;
    let max_attempts = 5;
    let mut refreshed_data = None;
    
    while attempts < max_attempts {
        if let Some(data) = cache.get(query_key).await {
            if data.data == format!("refreshed_{}", query_key) {
                refreshed_data = Some(data);
                break;
            }
        }
        sleep(Duration::from_millis(100)).await;
        attempts += 1;
    }
    
    let refreshed_data = refreshed_data.expect("Should have refreshed data");
    assert_eq!(refreshed_data.data, format!("refreshed_{}", query_key), 
        "Data should have been refreshed with new value");
} 
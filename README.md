# Communex API Client

This is a Rust implementation of the Communex API client, optimized for performance and designed to work asynchronously. The library provides methods for interacting with the blockchain via RPC commands through a FastAPI interface.

## Features

- **Query Map Caching**: Efficient caching layer with configurable TTL and background refresh worker.
- **Metrics**: Track cache hits, misses, and refresh failures.
- **Asynchronous Operations**: Designed to handle batch requests efficiently.
- **Comprehensive Test Coverage**: Includes tests for types, RPC client, query map, and error handling.

## Installation

To use this library, add the following to your `Cargo.toml`:

```toml
[dependencies]
comx-api = { git = "https://github.com/your-repo/comx-api.git" }
```

## Usage

### Query Map Cache

The `QueryMapCache` provides a caching layer for query results with a configurable time-to-live (TTL) and background refresh capabilities.

```rust
use comx_api::cache::{QueryMapCache, CacheConfig, QueryResult};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let config = CacheConfig {
        ttl: Duration::from_secs(60),
        refresh_interval: Duration::from_secs(300),
        max_entries: 1000,
    };
    
    let cache = QueryMapCache::new(config);
    let query_key = "example_query";
    let result = QueryResult::new("example_data");
    
    cache.set(query_key, result).await;
    if let Some(cached_result) = cache.get(query_key).await {
        println!("Cached data: {}", cached_result.data);
    }
}
```

### Query Map

The `QueryMap` provides high-level access to blockchain state queries with caching support. It automatically handles RPC communication and response parsing.

```rust
use comx_api::rpc::RpcClient;
use comx_api::query_map::{QueryMap, QueryMapConfig};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let client = RpcClient::new("http://your-node-url");
    let config = QueryMapConfig {
        refresh_interval: Duration::from_secs(300),
        cache_duration: Duration::from_secs(600),
    };
    
    let query_map = QueryMap::new(client, config).unwrap();
    // Use the query map...
}
```

### Wallet Operations

The `WalletClient` provides methods for managing transfers, staking, and transaction tracking:

```rust
use comx_api::wallet::{WalletClient, TransferRequest, StakeRequest, TransactionStatus};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let client = WalletClient::new("http://your-node-url");
    // Use the wallet client...
}
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

This project is licensed under the MIT License.



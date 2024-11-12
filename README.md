# Communex Rust API Client

A Rust implementation of the Communex blockchain client API, optimized for high performance and async operations.

## Features

- Async RPC client with configurable retry mechanism
- Batch request support for efficient operations
- Query Map for blockchain state queries with validation
- SR25519 cryptographic operations using Substrate primitives
- Comprehensive error handling and response validation
- Full test coverage with mocked responses
- Type-safe balance and address handling

## Installation

Add to your Cargo.toml:

```toml
[dependencies]
comx-api = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use comx_api::{
    QueryMap, 
    QueryMapConfig,
    rpc::{RpcClient, RpcClientConfig},
    types::{Address, Balance}
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure RPC client
    let rpc_config = RpcClientConfig {
        timeout: Duration::from_secs(30),
        max_retries: 3,
    };
    let client = RpcClient::new_with_config("http://your-node-url", rpc_config);
    
    // Configure and create QueryMap
    let query_config = QueryMapConfig {
        refresh_interval: Duration::from_secs(300), // 5 minutes
        cache_duration: Duration::from_secs(600),   // 10 minutes
    };
    let query_map = QueryMap::new(client, query_config)?;
    
    // Query single balance
    let address = Address::from_str("cmx1abc...")?;
    let balance = query_map.get_balance(&address).await?;
    println!("Balance: {} {}", balance.amount()?, balance.denom());
    
    // Batch balance query
    let addresses = vec![
        Address::from_str("cmx1abc...")?,
        Address::from_str("cmx1def...")?
    ];
    let balances = query_map.get_balances(&addresses).await?;
    
    // Query stake relationships
    let stake_from = query_map.get_stake_from(&address).await?;
    let stake_to = query_map.get_stake_to(&address).await?;
    
    Ok(())
}
```

## RPC Operations

```rust
use comx_api::rpc::{RpcClient, BatchRequest};
use serde_json::json;

// Create RPC client with default config
let client = RpcClient::new("http://your-node-url");

// Single request with retry handling
let response = client.request(
    "query_balance",
    json!({
        "address": "cmx1abc..."
    })
).await?;

// Batch request
let mut batch = BatchRequest::new();
batch.add_request("query_balance", json!({"address": "cmx1abc..."}));
batch.add_request("query_balance", json!({"address": "cmx1def..."}));
let responses = client.batch_request(batch).await?;
```

## Error Handling

```rust
use comx_api::error::CommunexError;

match query_map.get_balance(&address).await {
    Ok(balance) => println!("Balance: {}", balance),
    Err(CommunexError::ConnectionError(msg)) => eprintln!("Connection failed: {}", msg),
    Err(CommunexError::RpcError { code, message }) => eprintln!("RPC error {}: {}", code, message),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Development

### Prerequisites

- Rust 1.70 or higher
- Cargo

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test
```

## Project Structure

```
src/
├── error/      # Error types and handling
├── rpc/        # RPC client implementation
│   ├── client.rs
│   └── batch.rs
├── query_map/  # Query Map implementation
│   ├── config.rs
│   └── query_map.rs
├── types/      # Core types
│   ├── address.rs
│   ├── balance.rs
│   └── transaction.rs
└── lib.rs      # Library root
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

[MIT](LICENSE)

## Acknowledgments

- Built using Substrate primitives for cryptographic operations
- Compatible with Communex blockchain ecosystem



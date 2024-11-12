# Communex Rust API Client

A Rust implementation of the Communex blockchain client API, optimized for high performance and async operations.

## Features

- SR25519 cryptographic operations using Substrate primitives
- Async RPC client with batch request support (requires tokio runtime)
- Query Map for efficient blockchain state queries
- Proper error handling and response validation
- Full test coverage with mocked responses

## Installation

Add to your Cargo.toml:

```toml
[dependencies]
comx-api = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use comx_api::{QueryMap, QueryMapConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create QueryMap with default configuration
    let config = QueryMapConfig {
        refresh_interval: Duration::from_secs(300), // 5 minutes
        cache_duration: Duration::from_secs(600),   // 10 minutes
    };
    
    let query_map = QueryMap::new(client, config)?;
    
    // Query balance
    let balance = query_map.get_balance("cmx1abc...").await?;
    println!("Balance: {} {}", balance.amount()?, balance.denom());
    
    // Batch balance query
    let addresses = vec!["cmx1abc...", "cmx1def..."];
    let balances = query_map.get_balances(&addresses).await?;
    
    // Query stake relationships
    let stake_from = query_map.get_stake_from("cmx1abc...").await?;
    let stake_to = query_map.get_stake_to("cmx1abc...").await?;
    
    Ok(())
}
```

## RPC Operations

```rust
use comx_api::rpc::{RpcClient, BatchRequest};
use serde_json::json;

// Create RPC client
let client = RpcClient::new("http://your-node-url");

// Single request
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
├── query_map/  # Query Map implementation
├── types/      # Core types (Address, Balance, etc.)
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



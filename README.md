# Communex API Client

A high-performance, asynchronous Rust implementation for interacting with the Communex blockchain through RPC commands. This library provides robust caching, comprehensive error handling, and support for wallet operations.

## Features

- **Asynchronous Operations**: Built on tokio for efficient async/await support
- **Query Map Caching**: 
  - Configurable TTL and background refresh
  - Automatic cache invalidation
  - Performance metrics tracking
- **Wallet Operations**:
  - Transaction management
  - Balance queries
  - Staking operations
  - Transaction status tracking
- **Cryptographic Security**:
  - SR25519 key management
  - Transaction signing and verification
  - Address derivation
- **RPC Client**:
  - Automatic retry mechanism
  - Batch request support
  - Configurable timeouts
  - Comprehensive error handling

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
comx-api = { git = "https://github.com/your-repo/comx-api.git" }
```

## Usage

### Wallet Operations

```rust
use comx_api::wallet::{WalletClient, TransferRequest};
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Create client with custom timeout
    let client = WalletClient::with_timeout(
        "http://your-node-url",
        Duration::from_secs(30)
    );
    
    // Single transfer
    let transfer = TransferRequest {
        from: "cmx1sender...".into(),
        to: "cmx1receiver...".into(),
        amount: 1000,
        denom: "COMAI".into(),
    };
    
    let result = client.transfer(transfer).await?;

    // Batch transfer
    let transfers = vec![
        TransferRequest {
            from: "cmx1sender...".into(),
            to: "cmx1receiver1...".into(),
            amount: 1000,
            denom: "COMAI".into(),
        },
        TransferRequest {
            from: "cmx1sender...".into(),
            to: "cmx1receiver2...".into(),
            amount: 2000,
            denom: "COMAI".into(),
        },
    ];
    
    let batch_result = client.batch_transfer(transfers).await?;
}
```

### Staking Operations

```rust
use comx_api::wallet::{WalletClient, staking::StakeRequest};

#[tokio::main]
async fn main() {
    let client = WalletClient::new("http://your-node-url");
    
    // Stake tokens
    let stake = StakeRequest {
        from: "cmx1sender...".into(),
        amount: 1000,
        denom: "COMAI".into(),
    };
    
    let result = client.stake(stake).await?;
}
```

### Query Map Cache

```rust
use comx_api::cache::{QueryMapCache, CacheConfig};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let config = CacheConfig {
        ttl: Duration::from_secs(60),
        refresh_interval: Duration::from_secs(300),
        max_entries: 1000,
    };
    
    let cache = QueryMapCache::new(config);
    cache.start_background_refresh().await;
}
```

## Running the Program

To execute the program, ensure you have the Rust toolchain installed. Run the following command to start the application:

```bash
cargo run
```

### Testing and Benchmarking

Run the tests to ensure everything is working as expected:

```bash
cargo test
```

To execute benchmarks, use:

```bash
cargo bench
```

Ensure that all dependencies are installed and up-to-date before running these commands.

## Error Handling

The library provides comprehensive error handling through the `CommunexError` enum, covering:
- RPC communication errors
- Transaction validation
- Address formatting
- Cryptographic operations
- Cache operations
- Configuration validation

## Testing

The module client includes comprehensive test coverage:

- Unit tests for all core functionality
- Integration tests for API interactions
- Mock server tests for HTTP interactions
- Retry mechanism validation
- Error handling scenarios
- Rate limiting tests
- Cache behavior tests

Test patterns used:
- Wiremock for HTTP mocking
- Sequence-based response patterns
- Timeout and retry scenarios
- Edge case validation

To run tests:
```bash
cargo test
```

All tests are properly documented and follow best practices for async testing in Rust.

## License

MIT License - See LICENSE file for details.

## Contributing

Contributions are welcome! Please check the PROGRESS.md file for current development status and planned features.

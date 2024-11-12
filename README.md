# Communex Rust API Client

A Rust implementation of the Communex blockchain client API, optimized for high performance and async operations.

## Features

- SR25519 cryptographic operations using Substrate primitives
- Async RPC client with batch request support (requires tokio runtime)
- Wallet management and transaction signing
- Query map caching with automatic updates
- Compatible with existing Communex ecosystem

## Installation

Add to your Cargo.toml:

```toml
[dependencies]
comx-api = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use comx_api::{KeyPair, Transaction};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
// Create a keypair from seed phrase
    let keypair = KeyPair::from_seed_phrase(
    "wait swarm general shield hope target rebuild profit later pepper under hunt"
)?;
// Create and sign a transaction
let tx = Transaction::new(
    keypair.ss58_address(),
    "destination_address",
    "1000000",
    "COMAI",
    "transfer tokens"
);
let signed_tx = tx.sign(&keypair)?;
Ok(())
}
```

## Usage

### Key Management

```rust
use comx_api::KeyPair;

// Create from seed phrase
let keypair = KeyPair::from_seed_phrase("your seed phrase")?;

// Get SS58 address
let address = keypair.ss58_address();

// Derive child address
let derived = keypair.derive_address(0)?;
```

### Transaction Operations

```rust
use comx_api::Transaction;

// Create transaction
let tx = Transaction::new(
    from_address,
    to_address,
    amount,
    denom,
    memo
);

// Sign transaction
let signed = tx.sign(&keypair)?;

// Verify signature
assert!(signed.verify_signature().is_ok());
```

### RPC Operations

```rust
use comx_api::rpc::{RpcClient, BatchRequest};
use serde_json::json;

// Create RPC client
let client = RpcClient::new("http://your-node-url");

// Single request
let balance = client.request(
    "query_balance",
    json!({
        "address": "cmx1abc123..."
    })
).await?;

// Batch request
let mut batch = BatchRequest::new();
batch.add_request("query_balance", json!({"address": "cmx1abc123..."}));
batch.add_request("query_balance", json!({"address": "cmx1def456..."}));
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

### Running Examples

```bash
cargo run --example basic_transfer
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

[MIT](LICENSE)

## Acknowledgments

- Built using Substrate primitives for cryptographic operations
- Compatible with Communex blockchain ecosystem



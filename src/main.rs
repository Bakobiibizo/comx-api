use comx_api::modules::client::{ModuleClient, ModuleClientConfig};
use comx_api::crypto::KeyPair;
use std::env;

fn main() {
    // Example: Initialize a ModuleClient and perform a basic operation
    let keypair = KeyPair::generate();
    let config = ModuleClientConfig {
        host: "https://api.communex.io".to_string(),
        port: 443,
        max_retries: 3,
        timeout: std::time::Duration::from_secs(10),
    };
    let client = ModuleClient::with_config(config, keypair);

    println!("Client initialized with address: {}", client.keypair.address());
    // Add more client operations here as needed
}

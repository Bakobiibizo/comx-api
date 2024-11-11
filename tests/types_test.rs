use comx_api::{
    types::{Address, Balance, Transaction, SignedTransaction, RpcRequest},
    crypto::KeyPair,
};
use serde_json::{json, Value};
use std::fs;

#[test]
fn test_address_validation() {
    // Test valid address format
    let valid_address = "cmx1abc123..."; // Use actual address format from reference
    assert!(Address::new(valid_address).is_ok());

    // Test invalid address format
    let invalid_address = "invalid_address";
    assert!(Address::new(invalid_address).is_err());
}

#[test]
fn test_rpc_request_serialization() {
    let request = RpcRequest::new(
        "query_balance",
        json!({
            "address": "cmx1abc123...",
            "denom": "ucmx"
        }),
    );

    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("query_balance"));
    assert!(serialized.contains("jsonrpc"));
    assert!(serialized.contains("2.0"));
}

#[test]
fn test_balance_parsing() {
    let balance_json = json!({
        "amount": "1000000",
        "denom": "ucmx"
    });

    let balance: Balance = serde_json::from_value(balance_json).unwrap();
    assert_eq!(balance.amount(), 1000000);
    assert_eq!(balance.denom(), "ucmx");
}

#[test]
fn test_transaction_creation() {
    let tx = Transaction::new(
        "cmx1sender...",
        "cmx1receiver...",
        "1000000",
        "ucmx",
        "transfer tokens",
    );

    assert!(tx.validate().is_ok());
    assert_eq!(tx.amount(), "1000000");
    assert_eq!(tx.denom(), "ucmx");
}

#[test]
fn test_keypair_creation_and_validation() {
    // Test creating from seed phrase
    let seed_phrase = "test test test test test test test test test test test junk";
    let keypair = KeyPair::from_seed_phrase(seed_phrase).unwrap();
    
    // Verify SS58 address format
    assert!(keypair.ss58_address().starts_with("cmx1"));
    
    // Test public key accessibility
    let public_key = keypair.public_key();
    assert!(!public_key.is_empty());
    
    // Test invalid seed phrase
    let invalid_seed = "invalid seed phrase";
    assert!(KeyPair::from_seed_phrase(invalid_seed).is_err());
}

#[test]
fn test_transaction_signing() {
    let seed_phrase = "test test test test test test test test test test test junk";
    let keypair = KeyPair::from_seed_phrase(seed_phrase).unwrap();
    
    let tx = Transaction::new(
        keypair.ss58_address(),
        "cmx1receiver...",
        "1000000",
        "ucmx",
        "transfer tokens",
    );
    
    // Sign the transaction
    let signed_tx = tx.sign(&keypair).unwrap();
    
    // Verify signature
    assert!(signed_tx.verify_signature().is_ok());
    
    // Test invalid signature
    let different_seed = "differ test test test test test test test test test test junk";
    let different_keypair = KeyPair::from_seed_phrase(different_seed).unwrap();
    
    // Convert public key to fixed-size array
    let public_key: [u8; 32] = different_keypair.public_key()
        .try_into()
        .expect("Invalid public key length");
        
    assert!(signed_tx.verify_signature_with_key(&public_key).is_err());
}

#[test]
fn test_keypair_address_derivation() {
    let seed_phrase = "test test test test test test test test test test test junk";
    let keypair = KeyPair::from_seed_phrase(seed_phrase).unwrap();
    
    // Test address derivation for different paths
    let default_address = keypair.derive_address(0).unwrap();
    let second_address = keypair.derive_address(1).unwrap();
    
    assert_ne!(default_address, second_address);
    assert!(default_address.starts_with("cmx1"));
    assert!(second_address.starts_with("cmx1"));
}

#[test]
fn test_keypair_from_testkey() {
    // Read test key file
    let key_data = fs::read_to_string(format!("{}/.commune/key/testkey.json", env!("HOME")))
        .expect("Should read test key file");
    let key_json: Value = serde_json::from_str(&key_data).expect("Should parse JSON");
    let key_data: Value = serde_json::from_str(key_json["data"].as_str().unwrap()).expect("Should parse inner JSON");
    
    // Extract mnemonic
    let mnemonic = key_data["mnemonic"].as_str().unwrap();
    let expected_ss58 = key_data["ss58_address"].as_str().unwrap();
    
    // Create keypair from mnemonic
    let keypair = KeyPair::from_seed_phrase(mnemonic).unwrap();
    
    // Verify SS58 address matches
    assert_eq!(keypair.ss58_address(), expected_ss58);
    
    // Test signing
    let tx = Transaction::new(
        keypair.ss58_address(),
        "5CfjkoBAQ2LvJRmdcsoWXKSZkzR4k2KvpDVf2u1ohgm3UczR",
        "1000000",
        "ucmx",
        "test transfer",
    );
    
    let signed_tx = tx.sign(&keypair).unwrap();
    assert!(signed_tx.verify_signature().is_ok());
}

#[test]
fn test_transaction_serialization() {
    let tx = Transaction::new(
        "cmx1sender...",
        "cmx1receiver...",
        "1000000",
        "ucmx",
        "transfer tokens",
    );

    let serialized = serde_json::to_string(&tx).unwrap();
    let deserialized: Transaction = serde_json::from_str(&serialized).unwrap();

    assert_eq!(tx.amount(), deserialized.amount());
    assert_eq!(tx.denom(), deserialized.denom());
}

#[test]
fn test_signed_transaction_serialization() {
    let seed_phrase = "test test test test test test test test test test test junk";
    let keypair = KeyPair::from_seed_phrase(seed_phrase).unwrap();
    
    let tx = Transaction::new(
        keypair.ss58_address(),
        "cmx1receiver...",
        "1000000",
        "ucmx",
        "transfer tokens",
    );
    
    let signed_tx = tx.sign(&keypair).unwrap();
    let serialized = serde_json::to_string(&signed_tx).unwrap();
    let deserialized: SignedTransaction = serde_json::from_str(&serialized).unwrap();
    
    assert!(deserialized.verify_signature().is_ok());
}
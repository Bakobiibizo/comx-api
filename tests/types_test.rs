use comx_api::{
    types::{Address, Balance, Transaction, SignedTransaction, RpcRequest},
    crypto::KeyPair,
};
use serde_json::json;

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
            "denom": "COMAI"
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
        "denom": "COMAI"
    });

    let balance: Balance = serde_json::from_value(balance_json).unwrap();
    assert_eq!(balance.amount(), 1000000);
    assert_eq!(balance.denom(), "COMAI");
}

#[test]
fn test_transaction_creation() {
    let tx = Transaction::new(
        "cmx1sender...",
        "cmx1receiver...",
        "1000000",
        "COMAI",
        "transfer tokens",
    );

    assert!(tx.validate().is_ok());
    assert_eq!(tx.amount(), "1000000");
    assert_eq!(tx.denom(), "COMAI");
}

#[test]
fn test_keypair_creation_and_validation() {
    // Test creating from seed phrase
    let seed_phrase = "wait swarm general shield hope target rebuild profit later pepper under hunt"; //testkey.json
    let keypair = KeyPair::from_seed_phrase(seed_phrase).unwrap();
    
    // Verify SS58 address format
    assert!(keypair.ss58_address().starts_with("5"));
    
    // Test public key accessibility
    let public_key = keypair.public_key();
    assert!(!public_key.is_empty());
    
    // Test invalid seed phrase
    let invalid_seed = "invalid seed phrase";
    assert!(KeyPair::from_seed_phrase(invalid_seed).is_err());
}

#[test]
fn test_transaction_signing() {
    let seed_phrase = "wait swarm general shield hope target rebuild profit later pepper under hunt";
    let keypair = KeyPair::from_seed_phrase(seed_phrase).unwrap();
    
    let tx = Transaction::new(
        keypair.ss58_address(),
        "cmx1receiver...",
        "1000000",
        "COMAI",
        "transfer tokens",
    );
    
    // Sign the transaction
    let signed_tx = tx.sign(&keypair).unwrap();
    
    // Verify signature
    assert!(signed_tx.verify_signature().is_ok());
    
    // Test invalid signature
    let different_seed = "field mistake sustain bench foster cactus anxiety until riot capable obscure service"; // Test key2.json
    let different_keypair = KeyPair::from_seed_phrase(different_seed).unwrap();
    
    // Convert public key to fixed-size array
    let public_key: [u8; 32] = different_keypair.public_key()
        .try_into()
        .expect("Invalid public key length");
        
    assert!(signed_tx.verify_signature_with_key(&public_key).is_err());
}

#[test]
fn test_keypair_address_derivation() {
    let seed_phrase = "wait swarm general shield hope target rebuild profit later pepper under hunt";
    let keypair = KeyPair::from_seed_phrase(seed_phrase).unwrap();
    
    // Test address derivation for different paths
    let default_address = keypair.derive_address(0).unwrap();
    let second_address = keypair.derive_address(1).unwrap();
    
    assert_ne!(default_address, second_address);
    assert!(default_address.starts_with("5"));
    assert!(second_address.starts_with("5"));
}

#[test]
fn test_keypair_from_testkey() {
    let phrase = "wait swarm general shield hope target rebuild profit later pepper under hunt";
    println!("mnemonic: {}", phrase);
    
    let keypair = KeyPair::from_seed_phrase(phrase).unwrap();
    
    // Print intermediate values
    println!("Public key: {:?}", keypair.public_key());
    println!("Generated address: {}", keypair.ss58_address());
    println!("Expected address: 5CfjkoBAQ2LvJRmdcsoWXKSZkzR4k2KvpDVf2u1ohgm3UczR");
    
    assert_eq!(
        keypair.ss58_address(),
        "5CfjkoBAQ2LvJRmdcsoWXKSZkzR4k2KvpDVf2u1ohgm3UczR"
    );
}

#[test]
fn test_transaction_serialization() {
    let tx = Transaction::new(
        "cmx1sender...",
        "cmx1receiver...",
        "1000000",
        "COMAI",
        "transfer tokens",
    );

    let serialized = serde_json::to_string(&tx).unwrap();
    let deserialized: Transaction = serde_json::from_str(&serialized).unwrap();

    assert_eq!(tx.amount(), deserialized.amount());
    assert_eq!(tx.denom(), deserialized.denom());
}

#[test]
fn test_signed_transaction_serialization() {
    let seed_phrase = "wait swarm general shield hope target rebuild profit later pepper under hunt";
    let keypair = KeyPair::from_seed_phrase(seed_phrase).unwrap();
    
    let tx = Transaction::new(
        keypair.ss58_address(),
        "cmx1receiver...",
        "1000000",
        "COMAI",
        "transfer tokens",
    );
    
    let signed_tx = tx.sign(&keypair).unwrap();
    let serialized = serde_json::to_string(&signed_tx).unwrap();
    let deserialized: SignedTransaction = serde_json::from_str(&serialized).unwrap();
    
    assert!(deserialized.verify_signature().is_ok());
}
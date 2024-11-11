use serde::{Deserialize, Serialize};
use thiserror::Error;
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer};
use bip39::{Mnemonic, Language, Seed};
use blake2b_simd::blake2b;
use base58;

#[derive(Debug, Error)]
pub enum CommunexError {
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    #[error("Invalid seed phrase: {0}")]
    InvalidSeedPhrase(String),
    #[error("Signing error: {0}")]
    SigningError(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Key derivation error: {0}")]
    KeyDerivationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Address(String);

impl Address {
    pub fn new(address: impl Into<String>) -> Result<Self, CommunexError> {
        let address = address.into();
        // Basic validation: should start with "cmx1" and be of proper length
        if !address.starts_with("cmx1") || address.len() < 8 {
            return Err(CommunexError::InvalidAddress(address));
        }
        Ok(Self(address))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    amount: String,
    denom: String,
}

impl Balance {
    pub fn new(amount: impl Into<String>, denom: impl Into<String>) -> Self {
        Self {
            amount: amount.into(),
            denom: denom.into(),
        }
    }

    pub fn amount(&self) -> u64 {
        self.amount.parse().unwrap_or(0)
    }

    pub fn denom(&self) -> &str {
        &self.denom
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    from: String,
    to: String,
    amount: String,
    denom: String,
    memo: String,
    signature: Option<Vec<u8>>,
    public_key: Option<Vec<u8>>,
}

impl Transaction {
    pub fn new(
        from: impl Into<String>,
        to: impl Into<String>,
        amount: impl Into<String>,
        denom: impl Into<String>,
        memo: impl Into<String>,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            amount: amount.into(),
            denom: denom.into(),
            memo: memo.into(),
            signature: None,
            public_key: None,
        }
    }

    pub fn validate(&self) -> Result<(), CommunexError> {
        // Basic validation
        if !self.from.starts_with("cmx1") || !self.to.starts_with("cmx1") {
            return Err(CommunexError::InvalidTransaction(
                "Invalid address format".into(),
            ));
        }
        Ok(())
    }

    pub fn amount(&self) -> &str {
        &self.amount
    }

    pub fn denom(&self) -> &str {
        &self.denom
    }

    pub fn sign(&self, keypair: &KeyPair) -> Result<SignedTransaction, CommunexError> {
        let message = self.serialize_for_signing()
            .map_err(|e| CommunexError::SigningError(e.to_string()))?;
        
        let signature = keypair.keypair.sign(&message);
        
        Ok(SignedTransaction {
            transaction: self.clone(),
            signature: signature.to_bytes().to_vec(),
            public_key: keypair.public_key().to_vec(),
        })
    }
    
    fn serialize_for_signing(&self) -> Result<Vec<u8>, serde_json::Error> {
        // Create a canonical form for signing
        let signing_data = serde_json::json!({
            "from": self.from,
            "to": self.to,
            "amount": self.amount,
            "denom": self.denom,
            "memo": self.memo,
        });
        
        serde_json::to_vec(&signing_data)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    transaction: Transaction,
    signature: Vec<u8>,
    public_key: Vec<u8>,
}

impl SignedTransaction {
    pub fn verify_signature(&self) -> Result<(), CommunexError> {
        if let Some(ref public_key) = self.transaction.public_key {
            self.verify_signature_with_key(public_key)
        } else {
            Err(CommunexError::InvalidSignature("No public key available".into()))
        }
    }
    
    pub fn verify_signature_with_key(&self, public_key: &[u8]) -> Result<(), CommunexError> {
        let public_key = PublicKey::from_bytes(public_key)
            .map_err(|e| CommunexError::InvalidSignature(e.to_string()))?;
        
        let signature = Signature::from_bytes(&self.signature)
            .map_err(|e| CommunexError::InvalidSignature(e.to_string()))?;
        
        let message = self.transaction.serialize_for_signing()
            .map_err(|e| CommunexError::SigningError(e.to_string()))?;
        
        public_key.verify(&message, &signature)
            .map_err(|e| CommunexError::InvalidSignature(e.to_string()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: u64,
}

impl RpcRequest {
    pub fn new(method: impl Into<String>, params: serde_json::Value) -> Self {
        static mut REQUEST_ID: u64 = 0;
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id: unsafe { REQUEST_ID += 1; REQUEST_ID },
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<RpcError>,
    pub id: u64,
}

#[derive(Debug, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug)]
pub struct KeyPair {
    keypair: Keypair,
    ss58_address: String,
}

impl KeyPair {
    pub fn from_seed_phrase(phrase: &str) -> Result<Self, CommunexError> {
        let mnemonic = Mnemonic::from_phrase(phrase, Language::English)
            .map_err(|e| CommunexError::InvalidSeedPhrase(e.to_string()))?;
        
        let seed = Seed::new(&mnemonic, "");
        let seed_bytes = &seed.as_bytes()[..32];
        
        let secret = SecretKey::from_bytes(seed_bytes)
            .map_err(|e| CommunexError::KeyDerivationError(e.to_string()))?;
        let public = PublicKey::from(&secret);
        let keypair = Keypair { secret, public };
        
        let ss58_address = Self::generate_ss58_address(&public);
        
        Ok(Self {
            keypair,
            ss58_address,
        })
    }
    
    pub fn ss58_address(&self) -> &str {
        &self.ss58_address
    }
    
    pub fn public_key(&self) -> &[u8] {
        self.keypair.public.as_bytes()
    }
    
    pub fn derive_address(&self, index: u32) -> Result<String, CommunexError> {
        // Derive new key using hierarchical deterministic derivation
        let path = format!("m/44'/354'/{}'", index);
        let derived_key = self.derive_key(&path)
            .map_err(|e| CommunexError::KeyDerivationError(e.to_string()))?;
        
        Ok(Self::generate_ss58_address(&derived_key))
    }
    
    fn derive_key(&self, path: &str) -> Result<PublicKey, CommunexError> {
        // For now, we'll implement a simple derivation
        // In production, this should use proper HD key derivation
        let message = path.as_bytes();
        let signature = self.keypair.sign(message);
        let derived_bytes = &signature.to_bytes()[..32];
        
        PublicKey::from_bytes(derived_bytes)
            .map_err(|e| CommunexError::KeyDerivationError(e.to_string()))
    }
    
    fn generate_ss58_address(public_key: &PublicKey) -> String {
        let mut bytes = vec![42u8]; // SS58 prefix for substrate (42)
        bytes.extend_from_slice(public_key.as_bytes());
        let hash = blake2b(&bytes);
        let mut full_bytes = bytes;
        full_bytes.extend_from_slice(&hash.as_bytes()[0..2]);
        
        base58::encode(&full_bytes)
    }
}

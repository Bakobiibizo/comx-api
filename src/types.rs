use serde::{Deserialize, Serialize};
use ed25519_dalek::{Signature, VerifyingKey, SIGNATURE_LENGTH, PUBLIC_KEY_LENGTH, Verifier};
use crate::error::CommunexError;
use crate::crypto::{KeyPair, serde::{hex_signature, hex_pubkey}};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        
        let signature = keypair.sign(&message);
        
        Ok(SignedTransaction {
            transaction: self.clone(),
            signature: signature.to_bytes(),
            public_key: keypair.verifying_key.to_bytes(),
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
    #[serde(with = "hex_signature")]
    signature: [u8; SIGNATURE_LENGTH],
    #[serde(with = "hex_pubkey")]
    public_key: [u8; PUBLIC_KEY_LENGTH],
}

impl SignedTransaction {
    pub fn verify_signature(&self) -> Result<(), CommunexError> {
        self.verify_signature_with_key(&self.public_key)
    }
    
    pub fn verify_signature_with_key(&self, public_key: &[u8; PUBLIC_KEY_LENGTH]) -> Result<(), CommunexError> {
        let verifying_key = VerifyingKey::from_bytes(public_key)
            .map_err(|e| CommunexError::InvalidSignature(e.to_string()))?;
        
        let signature = Signature::from_bytes(&self.signature);
        
        let message = self.transaction.serialize_for_signing()
            .map_err(|e| CommunexError::SigningError(e.to_string()))?;
        
        verifying_key.verify(&message, &signature)
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

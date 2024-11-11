use serde::{Deserialize, Serialize};
use crate::error::CommunexError;
use crate::crypto::{KeyPair, serde::{hex_signature, hex_pubkey}};
use sp_core::sr25519::{Public, Signature, Pair};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
        let public_key = keypair.public_key();
        
        Ok(SignedTransaction {
            transaction: self.clone(),
            signature,
            public_key,
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
    signature: [u8; 64],
    #[serde(with = "hex_pubkey")]
    public_key: [u8; 32],
}

impl SignedTransaction {
    pub fn verify_signature(&self) -> Result<(), CommunexError> {
        self.verify_signature_with_key(&self.public_key)
    }
    
    pub fn verify_signature_with_key(&self, public_key: &[u8; 32]) -> Result<(), CommunexError> {
        let public = Public::from_raw(*public_key);
        let signature = Signature::from_raw(self.signature);
        
        let message = self.transaction.serialize_for_signing()
            .map_err(|e| CommunexError::SigningError(e.to_string()))?;
            
        if <Pair as sp_core::Pair>::verify(&signature, &message, &public) {
            Ok(())
        } else {
            Err(CommunexError::InvalidSignature("Signature verification failed".into()))
        }
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

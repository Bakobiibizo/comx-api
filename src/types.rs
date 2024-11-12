use serde::{Deserialize, Serialize};
use crate::error::CommunexError;
use crate::crypto::{KeyPair, serde::hex_bytes};
use sp_core::sr25519::{Public, Signature, Pair};
use sp_core::sr25519::{PUBLIC_KEY_SERIALIZED_SIZE, SIGNATURE_SERIALIZED_SIZE};
use std::fmt::Display;
use std::string::String;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use bs58;

static REQUEST_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address(String);

impl Address {
    pub fn new(address: impl Into<String>) -> Result<Self, CommunexError> {
        let address = address.into();
        if !address.starts_with("cmx1") {
            return Err(CommunexError::InvalidAddress(address));
        }
        // Validate base58 format
        if let Err(_) = bs58::decode(&address[4..]).into_vec() {
            return Err(CommunexError::InvalidAddress(address));
        }
        Ok(Self(address))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigUint(pub [u8; 32], pub u64);
impl std::fmt::Display for BigUint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hex::encode(&self.0))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    amount: String,
    denom: String,
}

impl Balance {
    pub fn new(amount: impl Into<String>, denom: impl Into<String>) -> Result<Self, CommunexError> {
        let amount = amount.into();
        let denom = denom.into();
        
        // Validate amount can be parsed as u64
        amount.parse::<u64>()
            .map_err(|_| CommunexError::InvalidAmount("Invalid amount format".into()))?;
            
        // Validate denomination
        if !is_valid_denom(&denom) {
            return Err(CommunexError::InvalidDenom(denom));
        }

        Ok(Self { amount, denom })
    }

    pub fn amount(&self) -> Result<u64, CommunexError> {
        self.amount
            .parse()
            .map_err(|_| CommunexError::InvalidAmount("Invalid amount format".into()))
    }

    pub fn denom(&self) -> &str {
        &self.denom
    }

    pub fn from_rpc(value: &Value) -> Result<Self, CommunexError> {
        let amount = value.get("amount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommunexError::MalformedResponse("Missing amount field".into()))?;
            
        let denom = value.get("denom")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommunexError::MalformedResponse("Missing denom field".into()))?;

        // Validate amount can be parsed as u64
        amount.parse::<u64>()
            .map_err(|_| CommunexError::InvalidAmount("Invalid amount format".into()))?;
            
        // Validate denomination
        if !is_valid_denom(denom) {
            return Err(CommunexError::InvalidDenom(denom.to_string()));
        }

        Ok(Self {
            amount: amount.to_string(),
            denom: denom.to_string(),
        })
    }
}

// Remove the parse() call on denom since we're not parsing it anymore
fn is_valid_denom(denom: &str) -> bool {
    const VALID_DENOMS: &[&str] = &["COMAI"];
    VALID_DENOMS.contains(&denom)
}

impl Display for Balance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.amount, self.denom)
    }
}

impl FromRpcResponse for Balance {
    fn from_rpc(value: Value) -> Result<Self, CommunexError> {
        // For RPC responses, we need to extract the result field
        let result = if let Some(result) = value.get("result") {
            result
        } else {
            &value
        };

        // Try to deserialize the balance
        serde_json::from_value(result.clone())
            .map_err(|e| CommunexError::ParseError(e.to_string()))
    }
}

#[derive(Serialize)]
struct SigningData<'a> {
    from: &'a str,
    to: &'a str,
    amount: &'a str,
    denom: &'a str,
    memo: &'a str,
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
        // Validate addresses
        if !self.from.starts_with("cmx1") || !self.to.starts_with("cmx1") {
            return Err(CommunexError::InvalidAddress("Invalid address format".into()));
        }

        // Validate amount is not zero
        match self.amount.parse::<u64>() {
            Ok(amount) if amount == 0 => {
                return Err(CommunexError::InvalidAmount("Amount cannot be zero".into()));
            }
            Err(_) => {
                return Err(CommunexError::InvalidAmount("Invalid amount format".into()));
            }
            _ => {}
        }

        // Validate denomination
        if !is_valid_denom(&self.denom) {
            return Err(CommunexError::InvalidDenom(self.denom.clone()));
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
        let signing_data = SigningData {
            from: &self.from,
            to: &self.to,
            amount: &self.amount,
            denom: &self.denom,
            memo: &self.memo,
        };
        serde_json::to_vec(&signing_data)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    #[serde(with = "hex_bytes")]
    pub signature: [u8; SIGNATURE_SERIALIZED_SIZE],
    #[serde(with = "hex_bytes")]
    pub public_key: [u8; PUBLIC_KEY_SERIALIZED_SIZE],
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
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id: REQUEST_ID.fetch_add(1, Ordering::Relaxed),
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

pub trait FromRpcResponse: Sized {
    fn from_rpc(value: Value) -> Result<Self, CommunexError>;
} 
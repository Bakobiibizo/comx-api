use thiserror::Error;
use std::cmp::PartialEq; 
use std::fmt;
use reqwest;

#[derive(Debug, Error, PartialEq)]
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
    
    #[error("RPC error: {code} - {message}")]
    RpcError {
        code: i32,
        message: String,
    },

    #[error("Batch RPC errors: {}", format_errors(.0))]
    BatchRpcError(Vec<RpcErrorDetail>),
    
    #[error("Malformed response: {0}")]
    MalformedResponse(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Communex error: {0}")]
    CommunexError(String),

    #[error("Invalid balance: {0}")]
    InvalidBalance(String),

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Invalid denomination: {0}")]
    InvalidDenom(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Request timeout: {0}")]
    RequestTimeout(String),

    #[error("Invalid Header: {0}")]
    InvalidHeader(String),
    
}

impl CommunexError {
    pub fn to_string(&self) -> String {
        format!("{}", self)
    }
}

#[derive(Debug, PartialEq)]
pub struct RpcErrorDetail {
    pub code: i32,
    pub message: String,
    pub request_id: Option<u32>,
}

impl fmt::Display for RpcErrorDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "code: {}, message: {}{}", 
            self.code, 
            self.message,
            self.request_id.map_or(String::new(), |id| format!(", request_id: {}", id))
        )
    }
}

fn format_errors(errors: &Vec<RpcErrorDetail>) -> String {
    errors.iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

impl From<reqwest::Error> for CommunexError {
    fn from(error: reqwest::Error) -> Self {
        CommunexError::ConnectionError(error.to_string())
    }
} 
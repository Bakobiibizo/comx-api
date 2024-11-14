use thiserror::Error;
use std::cmp::PartialEq; 
use std::fmt;
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
    
}

impl CommunexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Self::RpcError { code, message } => 
                write!(f, "RPC error {}: {}", code, message),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Self::InvalidAddress(msg) => write!(f, "Invalid address: {}", msg),
            Self::InvalidTransaction(msg) => write!(f, "Invalid transaction: {}", msg),
            Self::InvalidSeedPhrase(msg) => write!(f, "Invalid seed phrase: {}", msg),
            Self::SigningError(msg) => write!(f, "Signing error: {}", msg),
            Self::InvalidSignature(msg) => write!(f, "Invalid signature: {}", msg),
            Self::KeyDerivationError(msg) => write!(f, "Key derivation error: {}", msg),
            Self::MalformedResponse(msg) => write!(f, "Malformed response: {}", msg),
            Self::BatchRpcError(errors) => write!(f, "Batch RPC errors: {}", format_errors(errors)),
            Self::CommunexError(msg) => write!(f, "Communex error: {}", msg),
            Self::InvalidBalance(msg) => write!(f, "Invalid balance: {}", msg),
            Self::InvalidAmount(msg) => write!(f, "Invalid amount: {}", msg),
            Self::InvalidDenom(msg) => write!(f, "Invalid denomination: {}", msg),
            Self::RequestTimeout(msg) => write!(f, "Request timeout: {}", msg),
        }
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
use thiserror::Error;

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
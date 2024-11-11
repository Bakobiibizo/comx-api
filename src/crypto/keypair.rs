use ed25519_dalek::{
    SigningKey,
    VerifyingKey,
    SECRET_KEY_LENGTH,
    PUBLIC_KEY_LENGTH,
    Signer,
    Signature
};
use bip39::Mnemonic;
use blake2b_simd::blake2b;
use bs58;
use crate::error::CommunexError;

#[derive(Debug)]
pub struct KeyPair {
    pub(crate) signing_key: SigningKey,
    pub(crate) verifying_key: VerifyingKey,
    ss58_address: String,
}

impl KeyPair {
    pub fn from_seed_phrase(phrase: &str) -> Result<Self, CommunexError> {
        let mnemonic = Mnemonic::parse_normalized(phrase)
            .map_err(|e| CommunexError::InvalidSeedPhrase(e.to_string()))?;
        
        let seed = mnemonic.to_seed("");
        let seed_bytes: [u8; SECRET_KEY_LENGTH] = seed[..SECRET_KEY_LENGTH]
            .try_into()
            .map_err(|_| CommunexError::KeyDerivationError("Invalid seed length".into()))?;

        let signing_key = SigningKey::from_bytes(&seed_bytes);
        let verifying_key = signing_key.verifying_key();
        let ss58_address = Self::generate_ss58_address(&verifying_key);
        
        Ok(Self {
            signing_key,
            verifying_key,
            ss58_address,
        })
    }
    
    pub fn ss58_address(&self) -> &str {
        &self.ss58_address
    }
    
    pub fn public_key(&self) -> &[u8] {
        self.verifying_key.as_bytes()
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        Signer::sign(&self.signing_key, message)
    }

    pub fn derive_address(&self, index: u32) -> Result<String, CommunexError> {
        let message = index.to_le_bytes();
        let signature = self.sign(&message);
        let derived_bytes: [u8; PUBLIC_KEY_LENGTH] = signature.to_bytes()[..PUBLIC_KEY_LENGTH]
            .try_into()
            .map_err(|_| CommunexError::KeyDerivationError("Invalid derived key length".into()))?;
        
        let verifying_key = VerifyingKey::from_bytes(&derived_bytes)
            .map_err(|e| CommunexError::KeyDerivationError(e.to_string()))?;
        
        Ok(Self::generate_ss58_address(&verifying_key))
    }
    
    fn generate_ss58_address(key: &VerifyingKey) -> String {
        let mut bytes = vec![42u8]; // SS58 prefix for substrate
        bytes.extend_from_slice(key.as_bytes());
        let hash = blake2b(&bytes);
        let mut full_bytes = bytes;
        full_bytes.extend_from_slice(&hash.as_bytes()[0..2]);
        
        bs58::encode(full_bytes).into_string()
    }
}


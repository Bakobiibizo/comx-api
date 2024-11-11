use sp_core::{
    sr25519::{Pair, Signature},
    Pair as PairT,
    crypto::{Ss58Codec, Ss58AddressFormat, DeriveJunction},
    
};
use bip39::Mnemonic;
use crate::error::CommunexError;

pub struct KeyPair {
    pair: Pair,
    ss58_address: String,
}

impl KeyPair {
    pub fn from_seed_phrase(phrase: &str) -> Result<Self, CommunexError> {
        let (pair, _) = Pair::from_phrase(phrase, None)
            .map_err(|e| CommunexError::InvalidSeedPhrase(e.to_string()))?;
        let public = pair.public();
        let ss58_address = public.to_ss58check_with_version(Ss58AddressFormat::custom(42));
    
        Ok(Self {
            pair,
            ss58_address,
        })
    }
    

    pub fn ss58_address(&self) -> &str {
        &self.ss58_address
    }

    pub fn public_key(&self) -> [u8; 32] {
        self.pair.public().0
    }

    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.pair.sign(message).0
    }
    
    pub fn derive_address(&self, index: u32) -> Result<String, CommunexError> {
        // Create a hard derivation junction from the index
        let junction = DeriveJunction::hard(&index.to_le_bytes());
        
        // Derive new key pair using substrate's derivation
        let (derived_pair, _) = self.pair.derive(
            std::iter::once(junction),
            None
        ).map_err(|e| CommunexError::KeyDerivationError(e.to_string()))?;
        
        // Generate SS58 address for derived public key
        let public = derived_pair.public();
        Ok(public.to_ss58check_with_version(Ss58AddressFormat::custom(42)))
    }

    pub fn verify(&self, message: &[u8], signature: &[u8; 64]) -> bool {
        let sig = Signature::from_raw(*signature);
        Pair::verify(&sig, message, &self.pair.public())
    }
}


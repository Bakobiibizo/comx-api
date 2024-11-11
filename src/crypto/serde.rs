use serde::{Deserializer, Serializer};
use serde::de::Error;

pub mod hex_bytes {
    use super::*;

    pub fn serialize<const N: usize, S>(bytes: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, const N: usize, D>(deserializer: D) -> Result<[u8; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(|e| Error::custom(e.to_string()))?;
        bytes.try_into().map_err(|_| Error::custom(format!("Invalid length, expected {}", N)))
    }
}

// For backward compatibility
pub use hex_bytes as hex_signature;
pub use hex_bytes as hex_pubkey;

use serde::{Deserializer, Serializer};
use ed25519_dalek::{SIGNATURE_LENGTH, PUBLIC_KEY_LENGTH};

pub mod hex_signature {
    use super::*;

    pub fn serialize<S>(bytes: &[u8; SIGNATURE_LENGTH], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; SIGNATURE_LENGTH], D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(|e| Error::custom(e.to_string()))?;
        bytes.try_into().map_err(|_| Error::custom("Invalid signature length"))
    }
}

pub mod hex_pubkey {
    use super::*;

    pub fn serialize<S>(bytes: &[u8; PUBLIC_KEY_LENGTH], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; PUBLIC_KEY_LENGTH], D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(|e| Error::custom(e.to_string()))?;
        bytes.try_into().map_err(|_| Error::custom("Invalid public key length"))
    }
}

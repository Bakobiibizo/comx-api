#[macro_use]
extern crate log;

pub mod error;
pub mod types;
pub mod crypto;
pub mod rpc;
pub mod query_map;
pub mod cache;
pub mod wallet;
pub mod modules {
    pub mod client;
}

pub use error::CommunexError;
pub use types::{Address, Balance, Transaction, SignedTransaction};
pub use crypto::KeyPair;

#[cfg(test)]
mod tests {
    mod cache_tests;
}
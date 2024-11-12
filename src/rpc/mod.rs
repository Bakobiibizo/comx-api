mod rpc_client;
mod batch;

pub use rpc_client::RpcClient;
pub use batch::BatchRequest;
pub use crate::error::{CommunexError, RpcErrorDetail};

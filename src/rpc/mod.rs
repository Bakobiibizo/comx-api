mod rpc_client;

pub use rpc_client::RpcClient;
use serde_json::{Value, json};

#[derive(Debug)]
pub struct BatchRequest {
    pub requests: Vec<Value>,
}

impl BatchRequest {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    pub fn add_request(&mut self, method: &str, params: Value) {
        self.requests.push(json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": self.requests.len()
        }));
    }
}

#[derive(Debug)]
pub struct BatchResponse {
    pub successes: Vec<Value>,
    pub errors: Vec<RpcErrorDetail>,
}

#[derive(Debug)]
pub struct RpcErrorDetail {
    pub code: i32,
    pub message: String,
}

use serde_json::Value;

#[derive(Default)]
pub struct BatchRequest {
    requests: Vec<(String, Value)>,
}

impl BatchRequest {
    pub fn new() -> Self {
        Self { requests: Vec::new() }
    }

    pub fn add_request(&mut self, method: impl Into<String>, params: Value) {
        self.requests.push((method.into(), params));
    }

    pub(crate) fn into_requests(self) -> Vec<(String, Value)> {
        self.requests
    }
} 
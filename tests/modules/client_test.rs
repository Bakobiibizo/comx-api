use comx_api::{
    crypto::KeyPair,
    modules::client::{ModuleClient, ModuleClientConfig, ClientError},
};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicUsize;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestParams {
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestResponse {
    result: String,
}

#[tokio::test]
async fn test_module_client_successful_call() {
    // Start a mock server
    let mock_server = MockServer::start().await;
    
    // Create a test keypair
    let keypair = KeyPair::generate();
    
    // Configure client to use mock server
    let config = ModuleClientConfig {
        host: mock_server.uri(),
        port: 0, // Not needed for mock
        timeout: std::time::Duration::from_secs(1),
        max_retries: 1,
    };
    
    let client = ModuleClient::with_config(config, keypair.clone());
    
    // Set up the mock response
    Mock::given(method("POST"))
        .and(path("/test_method"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/test_method"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(TestResponse {
                result: "success".to_string(),
            }))
        .mount(&mock_server)
        .await;
    
    // Make the call
    let params = TestParams {
        value: "test".to_string(),
    };
    
    let result: TestResponse = client
        .call("test_method", &keypair.address(), params)
        .await
        .unwrap();
    
    assert_eq!(result.result, "success");
}

#[tokio::test]
async fn test_module_client_unauthorized() {
    let mock_server = MockServer::start().await;
    let keypair = KeyPair::generate();
    
    let config = ModuleClientConfig {
        host: mock_server.uri(),
        port: 0,
        timeout: std::time::Duration::from_secs(1),
        max_retries: 1,
    };
    
    let client = ModuleClient::with_config(config, keypair.clone());
    
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let params = TestParams {
        value: "test".to_string(),
    };
    
    let result = client
        .call::<_, TestResponse>("test_method", &keypair.address(), params)
        .await;
    
    assert!(matches!(result, Err(ClientError::Unauthorized)));
}

#[tokio::test]
async fn test_module_client_retry_success() {
    let mock_server = MockServer::start().await;
    let keypair = KeyPair::generate();
    
    let config = ModuleClientConfig {
        host: mock_server.uri(),
        port: 0,
        timeout: std::time::Duration::from_secs(1),
        max_retries: 2,
    };
    
    let client = ModuleClient::with_config(config, keypair.clone());
    
    // Set up mock to handle both requests with different responses based on sequence
    let _sequence_count = AtomicUsize::new(0);
    Mock::given(method("POST"))
        .and(path("/test_method"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;
    
    Mock::given(method("POST"))
        .and(path("/test_method"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(TestResponse {
                result: "success".to_string(),
            }))
        .mount(&mock_server)
        .await;

    let params = TestParams {
        value: "test".to_string(),
    };

    let result: TestResponse = client
        .call("test_method", &keypair.address(), params)
        .await
        .unwrap();

    assert_eq!(result.result, "success");
}

#[tokio::test]
async fn test_module_client_rate_limit() {
    let mock_server = MockServer::start().await;
    let keypair = KeyPair::generate();
    
    let config = ModuleClientConfig {
        host: mock_server.uri(),
        port: 0,
        timeout: std::time::Duration::from_secs(1),
        max_retries: 1,
    };
    
    let client = ModuleClient::with_config(config, keypair.clone());
    
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(429))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    let params = TestParams {
        value: "test".to_string(),
    };
    
    let result = client
        .call::<_, TestResponse>("test_method", &keypair.address(), params)
        .await;
    
    assert!(matches!(result, Err(ClientError::RateLimitExceeded)));
}

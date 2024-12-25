use comx_api::modules::client::{
    ModuleClient, ModuleClientConfig, EndpointConfig,
    AccessLevel, RateLimit,
};
use comx_api::crypto::KeyPair;
use std::time::Duration;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestParams {
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestResponse {
    result: String,
}

#[tokio::test]
async fn test_endpoint_configuration() {
    // Setup mock server
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(TestResponse {
                result: "success".to_string(),
            }))
        .mount(&mock_server)
        .await;

    // Create client with endpoint configuration
    let keypair = KeyPair::generate();
    let config = ModuleClientConfig {
        host: mock_server.uri(),
        port: 0,
        timeout: Duration::from_secs(5),
        max_retries: 3,
    };
    
    let mut client = ModuleClient::with_config(config, keypair);

    // Register test endpoint
    let endpoint_config = EndpointConfig {
        name: "test_method".to_string(),
        path: "/test".to_string(),
        access_level: AccessLevel::Protected,
        rate_limit: Some(RateLimit {
            max_requests: 100,
            window_secs: 60,
        }),
        timeout: Some(Duration::from_secs(30)),
        allow_retries: true,
        metadata: HashMap::new(),
    };
    client.register_endpoint(endpoint_config.clone());

    // Test endpoint retrieval
    let retrieved = client.get_endpoint("test_method").unwrap();
    assert_eq!(retrieved.name, "test_method");
    assert_eq!(retrieved.access_level, AccessLevel::Protected);

    // Test API call with registered endpoint
    let params = TestParams {
        value: "test".to_string(),
    };
    let response: TestResponse = client
        .call("test_method", "test_key", params)
        .await
        .unwrap();
    assert_eq!(response.result, "success");
}

#[tokio::test]
async fn test_endpoint_retry_disabled() {
    // Setup mock server
    let mock_server = MockServer::start().await;
    
    let mut response_count = std::sync::atomic::AtomicUsize::new(0);
    
    Mock::given(method("POST"))
        .and(path("/test"))
        .respond_with(move |_| {
            let count = response_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if count == 0 {
                ResponseTemplate::new(500)
                    .set_body_json(json!({ "error": "Server Error" }))
            } else {
                ResponseTemplate::new(200)
                    .set_body_json(TestResponse {
                        result: "success".to_string(),
                    })
            }
        })
        .expect(1) // Should only be called once since retries are disabled
        .mount(&mock_server)
        .await;

    // Create client with endpoint configuration
    let keypair = KeyPair::generate();
    let config = ModuleClientConfig {
        host: mock_server.uri(),
        port: 0,
        timeout: Duration::from_secs(5),
        max_retries: 3, // Client allows retries but endpoint disables them
    };
    
    let mut client = ModuleClient::with_config(config, keypair);

    // Register test endpoint with retries disabled
    let endpoint_config = EndpointConfig {
        name: "test_method".to_string(),
        path: "/test".to_string(),
        access_level: AccessLevel::Public,
        rate_limit: None,
        timeout: None,
        allow_retries: false, // Disable retries for this endpoint
        metadata: HashMap::new(),
    };
    client.register_endpoint(endpoint_config);

    // Test API call - should fail without retrying
    let params = TestParams {
        value: "test".to_string(),
    };
    let result = client
        .call::<_, TestResponse>("test_method", "test_key", params)
        .await;
    
    assert!(result.is_err());
}

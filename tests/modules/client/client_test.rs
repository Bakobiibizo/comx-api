use std::time::Duration;
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use communex::{
    crypto::sr25519::Keypair,
    modules::client::{ModuleClient, ModuleClientConfig, ClientError},
};

// Test request/response structures
#[derive(Debug, Serialize, Deserialize)]
struct TestRequest {
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestResponse {
    result: String,
}

// Test helpers
async fn setup_test_client() -> (MockServer, ModuleClient, Keypair) {
    let mock_server = MockServer::start().await;
    let keypair = Keypair::generate();
    
    let config = ModuleClientConfig {
        host: mock_server.uri().host().unwrap().to_string(),
        port: mock_server.uri().port().unwrap(),
        timeout: Duration::from_secs(1),
        max_retries: 1,
    };
    
    let client = ModuleClient::with_config(config, keypair.clone());
    (mock_server, client, keypair)
}

const TEST_TARGET_KEY: &str = "5EA6Dd3vejQco2FZomoAQgxacsTp7ZPFuR25TwxTiUKbkep1";

#[tokio::test]
async fn test_client_successful_request() {
    let (mock_server, client, _) = setup_test_client().await;

    Mock::given(method("POST"))
        .and(path("/test_method"))
        .and(header("X-Signature", wiremock::matchers::any()))
        .and(header("X-Key", wiremock::matchers::any()))
        .and(header("X-Crypto", "sr25519"))
        .and(header("X-Timestamp", wiremock::matchers::any()))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "result": "success"
            },
            "error": null
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let response = client
        .call::<TestRequest, TestResponse>(
            "test_method",
            TEST_TARGET_KEY,
            TestRequest {
                message: "test".to_string(),
            },
        )
        .await
        .unwrap();

    assert_eq!(response.result, "success");
}

#[tokio::test]
async fn test_client_timeout() {
    let (mock_server, client, _) = setup_test_client().await;

    Mock::given(method("POST"))
        .and(path("/test_method"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = client
        .call::<TestRequest, TestResponse>(
            "test_method",
            TEST_TARGET_KEY,
            TestRequest {
                message: "test".to_string(),
            },
        )
        .await;

    assert!(matches!(result, Err(ClientError::Timeout(_))));
}

#[tokio::test]
async fn test_client_unauthorized() {
    let (mock_server, client, _) = setup_test_client().await;

    Mock::given(method("POST"))
        .and(path("/test_method"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {
                "code": 401,
                "message": "Unauthorized access"
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = client
        .call::<TestRequest, TestResponse>(
            "test_method",
            TEST_TARGET_KEY,
            TestRequest {
                message: "test".to_string(),
            },
        )
        .await;

    assert!(matches!(result, Err(ClientError::Unauthorized)));
}

#[tokio::test]
async fn test_client_rate_limit() {
    let (mock_server, client, _) = setup_test_client().await;

    Mock::given(method("POST"))
        .and(path("/test_method"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "error": {
                "code": 429,
                "message": "Too many requests"
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = client
        .call::<TestRequest, TestResponse>(
            "test_method",
            TEST_TARGET_KEY,
            TestRequest {
                message: "test".to_string(),
            },
        )
        .await;

    assert!(matches!(result, Err(ClientError::RateLimitExceeded)));
}

#[tokio::test]
async fn test_client_method_not_found() {
    let (mock_server, client, _) = setup_test_client().await;

    Mock::given(method("POST"))
        .and(path("/nonexistent_method"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = client
        .call::<TestRequest, TestResponse>(
            "nonexistent_method",
            TEST_TARGET_KEY,
            TestRequest {
                message: "test".to_string(),
            },
        )
        .await;

    assert!(matches!(result, Err(ClientError::MethodNotFound(_))));
}

#[tokio::test]
async fn test_client_malformed_response() {
    let (mock_server, client, _) = setup_test_client().await;

    Mock::given(method("POST"))
        .and(path("/test_method"))
        .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = client
        .call::<TestRequest, TestResponse>(
            "test_method",
            TEST_TARGET_KEY,
            TestRequest {
                message: "test".to_string(),
            },
        )
        .await;

    assert!(matches!(result, Err(ClientError::SerializationError(_))));
}

#[tokio::test]
async fn test_client_server_error() {
    let (mock_server, client, _) = setup_test_client().await;

    Mock::given(method("POST"))
        .and(path("/test_method"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": {
                "code": 500,
                "message": "Internal server error"
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = client
        .call::<TestRequest, TestResponse>(
            "test_method",
            TEST_TARGET_KEY,
            TestRequest {
                message: "test".to_string(),
            },
        )
        .await;

    assert!(matches!(result, Err(ClientError::ServerError(_))));
}

#[tokio::test]
async fn test_client_retry_mechanism() {
    let (mock_server, client, _) = setup_test_client().await;

    // First request fails with 500, second succeeds
    Mock::given(method("POST"))
        .and(path("/test_method"))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/test_method"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "result": "success after retry"
            },
            "error": null
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let response = client
        .call::<TestRequest, TestResponse>(
            "test_method",
            TEST_TARGET_KEY,
            TestRequest {
                message: "test".to_string(),
            },
        )
        .await
        .unwrap();

    assert_eq!(response.result, "success after retry");
}

#[tokio::test]
async fn test_client_max_retries_exceeded() {
    let (mock_server, client, _) = setup_test_client().await;

    // All requests fail with 500
    Mock::given(method("POST"))
        .and(path("/test_method"))
        .respond_with(ResponseTemplate::new(500))
        .expect(2) // Initial request + 1 retry
        .mount(&mock_server)
        .await;

    let result = client
        .call::<TestRequest, TestResponse>(
            "test_method",
            TEST_TARGET_KEY,
            TestRequest {
                message: "test".to_string(),
            },
        )
        .await;

    assert!(matches!(result, Err(ClientError::MaxRetriesExceeded)));
}
  
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use comx_api::{
    crypto::KeyPair,
    modules::client::{ModuleClient, ModuleClientConfig},
    cache::{QueryMapCache, CacheConfig, QueryResult},
};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchParams {
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchResponse {
    result: String,
}

async fn setup_mock_server() -> MockServer {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/bench_method"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(BenchResponse {
                result: "success".to_string(),
            }))
        .mount(&mock_server)
        .await;
        
    mock_server
}

fn bench_module_client(c: &mut Criterion) {
    let mut group = c.benchmark_group("module_client");
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mock_server = rt.block_on(setup_mock_server());
    let keypair = KeyPair::generate();
    
    let config = ModuleClientConfig {
        host: mock_server.uri(),
        port: 0,
        timeout: Duration::from_secs(5),
        max_retries: 3,
    };
    
    let client = ModuleClient::with_config(config, keypair.clone());
    let params = BenchParams {
        value: "benchmark".to_string(),
    };

    // Benchmark basic call
    group.bench_function("basic_call", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(
                    client
                        .call::<_, BenchResponse>("bench_method", &keypair.address(), params.clone())
                        .await
                        .unwrap()
                )
            })
        })
    });

    // Benchmark signature generation
    group.bench_function("signature_gen", |b| {
        b.iter(|| {
            black_box(keypair.sign(b"test message"))
        })
    });

    // Benchmark signature verification
    let message = b"test message";
    let signature = keypair.sign(message);
    group.bench_function("signature_verify", |b| {
        b.iter(|| {
            black_box(keypair.verify(message, &signature))
        })
    });

    group.finish();
}

fn bench_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache");
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let cache = QueryMapCache::new(CacheConfig {
        ttl: Duration::from_secs(60),
        refresh_interval: Duration::from_secs(300),
        max_entries: 1000,
    });

    // Benchmark cache set operation
    group.bench_function("cache_set", |b| {
        let mut i = 0;
        b.iter(|| {
            let key = format!("key_{}", i);
            i += 1;
            rt.block_on(async {
                black_box(
                    cache.set(
                        &key,
                        QueryResult {
                            data: "test_value".to_string(),
                        }
                    ).await
                )
            })
        })
    });

    // Benchmark cache get operation
    group.bench_function("cache_get", |b| {
        let key = "test_key";
        rt.block_on(async {
            cache.set(
                key,
                QueryResult {
                    data: "test_value".to_string(),
                }
            ).await;
        });
        
        b.iter(|| {
            rt.block_on(async {
                black_box(
                    cache.get(key).await
                )
            })
        })
    });

    // Benchmark cache hit/miss ratio
    group.bench_function("cache_hit_miss", |b| {
        let mut i = 0;
        b.iter(|| {
            let key = format!("key_{}", i % 10); // Reuse 10 keys to test cache hits
            i += 1;
            rt.block_on(async {
                black_box(
                    if i % 3 == 0 { // Every third operation is a set
                        cache.set(
                            &key,
                            QueryResult {
                                data: "test_value".to_string(),
                            }
                        ).await;
                        true
                    } else {
                        cache.get(&key).await.is_some()
                    }
                )
            })
        })
    });

    group.finish();
}

criterion_group!(benches, bench_module_client, bench_cache);
criterion_main!(benches);

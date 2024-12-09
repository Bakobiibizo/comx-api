use comx_api::modules::client::{ModuleClient, ModuleClientConfig, EndpointConfig};
use comx_api::crypto::KeyPair;
use comx_api::wallet::{WalletClient, TransferRequest};
use actix_web::{web, App, HttpServer, HttpResponse, Responder, web::Data};
use actix_files as fs;
use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
struct CallParams {
    method: String,
    target_key: String,
    params: Value,
}

async fn list_endpoints(client: Data<Arc<Mutex<ModuleClient>>>) -> impl Responder {
    let client = client.lock().expect("Failed to lock ModuleClient");
    let endpoints: Vec<_> = client.endpoint_registry.list().into_iter().collect();
    HttpResponse::Ok().json(endpoints)
}

async fn register_endpoint(client: Data<Arc<Mutex<ModuleClient>>>, config: web::Json<EndpointConfig>) -> impl Responder {
    let mut client = client.lock().expect("Failed to lock ModuleClient");
    client.register_endpoint(config.into_inner());
    HttpResponse::Created().body("Endpoint registered")
}

async fn get_endpoint(client: Data<Arc<Mutex<ModuleClient>>>, name: web::Path<String>) -> impl Responder {
    let client = client.lock().expect("Failed to lock ModuleClient");
    if let Some(config) = client.get_endpoint(&name) {
        HttpResponse::Ok().json(config)
    } else {
        HttpResponse::NotFound().body("Endpoint not found")
    }
}

async fn call_method(client: Data<Arc<Mutex<ModuleClient>>>, call_params: web::Json<CallParams>) -> impl Responder {
    let client = client.lock().expect("Failed to lock ModuleClient");
    let CallParams { method, target_key, params } = call_params.into_inner();
    match client.call::<Value, Value>(&method, &target_key, params).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {:?}", e)),
    }
}

async fn get_balance(client: Data<Arc<WalletClient>>, address: web::Path<String>) -> impl Responder {
    match client.get_free_balance(&address).await {
        Ok(balance) => HttpResponse::Ok().body(format!("Balance: {}", balance)),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {:?}", e)),
    }
}

async fn transfer(client: Data<Arc<WalletClient>>, transfer_request: web::Json<TransferRequest>) -> impl Responder {
    match client.transfer(transfer_request.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {:?}", e)),
    }
}

async fn sign_transaction(_client: Data<Arc<Mutex<ModuleClient>>>, _transaction: web::Json<Value>) -> impl Responder {
    HttpResponse::Ok().body("Transaction signed")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let keypair = KeyPair::generate();
    let config = ModuleClientConfig {
        host: "http://localhost".to_string(),
        port: 8080,
        max_retries: 3,
        timeout: std::time::Duration::from_secs(10),
    };
    let client = Arc::new(Mutex::new(ModuleClient::with_config(config, keypair)));
    let wallet_client = Arc::new(WalletClient::new("http://localhost"));

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(client.clone()))
            .app_data(Data::new(wallet_client.clone()))
            .route("/endpoints", web::get().to(list_endpoints))
            .route("/endpoints", web::post().to(register_endpoint))
            .route("/endpoints/{name}", web::get().to(get_endpoint))
            .route("/calls", web::post().to(call_method))
            .route("/balance/{address}", web::get().to(get_balance))
            .route("/transfer", web::post().to(transfer))
            .route("/sign_transaction", web::post().to(sign_transaction))
            .service(fs::Files::new("/swagger", "static/swagger").index_file("index.html"))
            .service(fs::Files::new("/swagger-ui.css", "static/swagger").index_file("swagger-ui.css"))
            .service(fs::Files::new("/index.css", "static/swagger").index_file("index.css"))
            .service(fs::Files::new("/swagger-ui-bundle.js", "static/swagger").index_file("swagger-ui-bundle.js"))
            .service(fs::Files::new("/swagger-ui-standalone-preset.js", "static/swagger").index_file("swagger-ui-standalone-preset.js"))
            .service(fs::Files::new("/swagger-initializer.js", "static/swagger").index_file("swagger-initializer.js"))
            .service(fs::Files::new("/api-docs", ".").index_file("swagger.yaml"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

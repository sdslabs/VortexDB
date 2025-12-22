mod config;

use std::sync::Arc;

use config::ServerConfig;
use grpc::run_grpc_server;
use http::run_http_server;
use tracing::{Level, event, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();

    info!("Starting VortexDB unified server...");

    let config = ServerConfig::load_config()
        .inspect_err(|err| event!(Level::ERROR, "Failed to load config: {}", err))?;

    info!("Configuration loaded successfully");
    if !config.disable_http {
        info!("HTTP server will listen on: {}", config.http_addr);
    }
    info!("gRPC server will listen on: {}", config.grpc_addr);

    let vector_db = api::init_api(config.db_config)
        .inspect_err(|err| event!(Level::ERROR, "Failed to init API: {:?}", err))?;

    let shared_db = Arc::new(vector_db);
    info!("VectorDb initialized successfully");

    // Spawn HTTP server task if enabled
    let http_handle = if !config.disable_http {
        let db = Arc::clone(&shared_db);
        let addr = config.http_addr;
        Some(tokio::spawn(async move { run_http_server(db, addr).await }))
    } else {
        info!("HTTP server is disabled");
        None
    };

    // Spawn gRPC server task
    let grpc_handle = {
        let db = Arc::clone(&shared_db);
        let addr = config.grpc_addr;
        let password = config.grpc_root_password;
        let logging = config.logging;
        tokio::spawn(async move { run_grpc_server(db, addr, password, logging).await })
    };

    // Run servers concurrently
    if let Some(http) = http_handle {
        tokio::select! {
            result = http => {
                match result {
                    Err(e) => event!(Level::ERROR, "HTTP server task error: {}", e),
                    Ok(Err(e)) => event!(Level::ERROR, "HTTP server error: {}", e),
                    Ok(Ok(())) => {}
                }
            }
            result = grpc_handle => {
                match result {
                    Err(e) => event!(Level::ERROR, "gRPC server task error: {}", e),
                    Ok(Err(e)) => event!(Level::ERROR, "gRPC server error: {}", e),
                    Ok(Ok(())) => {}
                }
            }
        }
    } else {
        match grpc_handle.await {
            Err(e) => event!(Level::ERROR, "gRPC server task error: {}", e),
            Ok(Err(e)) => event!(Level::ERROR, "gRPC server error: {}", e),
            Ok(Ok(())) => {}
        }
    }

    Ok(())
}

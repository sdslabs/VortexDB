mod config;

use std::sync::Arc;

use config::ServerConfig;
use defs::BoxError;
use grpc::run_grpc_server;
use http::run_http_server;
use tokio::signal;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    tracing_subscriber::fmt::init();

    info!("Starting VortexDB unified server...");

    let config =
        ServerConfig::load_config().inspect_err(|err| error!("Failed to load config: {}", err))?;

    info!("Configuration loaded successfully");
    if !config.disable_http {
        info!("HTTP server will listen on: {}", config.http_addr);
    }
    info!("gRPC server will listen on: {}", config.grpc_addr);

    let vector_db = api::init_api(config.db_config)
        .inspect_err(|err| error!("Failed to init API: {:?}", err))?;

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

    if let Some(http) = http_handle {
        tokio::select! {
            _ = shutdown_signal() => {
                info!("Stopping servers");
            }
            result = http => {
                match result {
                    Err(e) => error!("HTTP server task error: {}", e),
                    Ok(Err(e)) => error!("HTTP server error: {}", e),
                    Ok(Ok(())) => {}
                }
            }
            result = grpc_handle => {
                match result {
                    Err(e) => error!("gRPC server task error: {}", e),
                    Ok(Err(e)) => error!("gRPC server error: {}", e),
                    Ok(Ok(())) => {}
                }
            }
        }
    } else {
        tokio::select! {
            _ = shutdown_signal() => {
                info!("Stopping servers");
            }
            result = grpc_handle => {
                match result {
                    Err(e) => error!("gRPC server task error: {}", e),
                    Ok(Err(e)) => error!("gRPC server error: {}", e),
                    Ok(Ok(())) => {}
                }
            }
        }
    }

    info!("Shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}

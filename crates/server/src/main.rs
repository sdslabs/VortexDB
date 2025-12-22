mod config;

use std::sync::Arc;

use config::ServerConfig;
use grpc_server::service::{VectorDBService, run_server};
use grpc_server::utils::ServerEndpoint;
use http_server::create_router;
use tokio::net::TcpListener;
use tracing::{Level, event, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting VortexDB unified server...");

    // Load configuration
    let config = ServerConfig::load_config()
        .inspect_err(|err| event!(Level::ERROR, "Failed to load config: {}", err))?;

    info!("Configuration loaded successfully");
    if !config.disable_http {
        info!("HTTP server will listen on: {}", config.http_addr);
    }
    info!("gRPC server will listen on: {}", config.grpc_addr);

    // Initialize the shared VectorDb instance
    let vector_db = api::init_api(config.db_config)
        .inspect_err(|err| event!(Level::ERROR, "Failed to init API: {:?}", err))?;

    let shared_db = Arc::new(vector_db);
    info!("VectorDb initialized successfully");

    // Create HTTP server task if enabled
    let http_handle = if !config.disable_http {
        let http_db = Arc::clone(&shared_db);
        let http_addr = config.http_addr;
        Some(tokio::spawn(async move {
            let app = create_router(http_db);
            let listener = TcpListener::bind(http_addr).await?;
            info!("HTTP server listening on http://{}", http_addr);
            axum::serve(listener, app.into_make_service()).await?;
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        }))
    } else {
        info!("HTTP server is disabled");
        None
    };

    // Create gRPC server task (always runs)
    let grpc_db = Arc::clone(&shared_db);
    let grpc_addr = config.grpc_addr;
    let grpc_root_password = config.grpc_root_password;
    let logging = config.logging;
    let grpc_handle = tokio::spawn(async move {
        let vector_db_service = VectorDBService::new(grpc_db, logging);
        info!("gRPC server listening on {}", grpc_addr);
        run_server(
            vector_db_service,
            ServerEndpoint::Address(grpc_addr),
            grpc_root_password,
        )
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })?;
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    });

    // Run servers concurrently
    // If any server fails, log the error
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
        // Only gRPC server running
        match grpc_handle.await {
            Err(e) => event!(Level::ERROR, "gRPC server task error: {}", e),
            Ok(Err(e)) => event!(Level::ERROR, "gRPC server error: {}", e),
            Ok(Ok(())) => {}
        }
    }

    Ok(())
}

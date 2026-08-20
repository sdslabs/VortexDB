pub mod constants;
pub mod error;
pub mod interceptors;
pub mod service;
pub mod utils;

use api::VectorDb;
use defs::BoxError;
use service::{VectorDBService, run_server};
use std::net::SocketAddr;
use std::sync::Arc;
use utils::ServerEndpoint;

/// Runs the gRPC server on the specified address.
pub async fn run_grpc_server(
    db: Arc<VectorDb>,
    addr: SocketAddr,
    root_password: String,
    logging: bool,
) -> Result<(), BoxError> {
    let vector_db_service = VectorDBService::new(db, logging);
    run_server(
        vector_db_service,
        ServerEndpoint::Address(addr),
        root_password,
    )
    .await
    .map_err(|e| -> BoxError { e.to_string().into() })
}

#[cfg(test)]
mod tests;

pub mod handler;

use api::VectorDb;
use axum::{
    Router,
    routing::{get, post},
};
use defs::BoxError;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

use handler::{
    delete_point_handler, get_point_handler, health_handler, insert_point_handler, root_handler,
    search_points_handler,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<VectorDb>,
}

/// Creates the HTTP router with all VectorDB routes.
pub fn create_router(db: Arc<VectorDb>) -> Router {
    let app_state = AppState { db };
    Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/points", post(insert_point_handler))
        .route(
            "/points/{id}",
            get(get_point_handler).delete(delete_point_handler),
        )
        .route("/points/search", post(search_points_handler))
        .with_state(app_state)
}

/// Runs the HTTP server on the specified address.
pub async fn run_http_server(db: Arc<VectorDb>, addr: SocketAddr) -> Result<(), BoxError> {
    let app = create_router(db);
    let listener = TcpListener::bind(addr).await?;
    info!("HTTP server listening on http://{}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

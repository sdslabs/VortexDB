pub mod auth;
pub mod constants;
pub mod handler;

use api::VectorDb;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router, middleware,
    routing::{delete, get, post},
};
use defs::BoxError;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

use handler::{
    batch_insert_handler, batch_search_handler, delete_point_handler, get_point_handler,
    health_handler, insert_point_handler, root_handler, search_points_handler,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<VectorDb>,
    pub keys: Arc<defs::ApiKeyStore>,
}

pub fn create_router(db: Arc<VectorDb>, keys: Arc<defs::ApiKeyStore>) -> Router {
    let app_state = AppState { db, keys };

    let public_routes = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler));

    let write_routes = Router::new()
        .route("/points", post(insert_point_handler))
        .route("/points/{id}", delete(delete_point_handler))
        .route("/points/batch", post(batch_insert_handler))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth::require_write_key,
        ));

    let read_routes = Router::new()
        .route("/points/{id}", get(get_point_handler))
        .route("/points/search", post(search_points_handler))
        .route("/points/search/batch", post(batch_search_handler))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth::require_read_key,
        ));

    public_routes
        .merge(write_routes)
        .merge(read_routes)
        .with_state(app_state)
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024)) // 50MB limit
}

/// Runs the HTTP server on the specified address.
pub async fn run_http_server(
    db: Arc<VectorDb>,
    addr: SocketAddr,
    keys: Arc<defs::ApiKeyStore>,
) -> Result<(), BoxError> {
    let app = create_router(db, keys);
    let listener = TcpListener::bind(addr).await?;
    info!("HTTP server listening on http://{}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[cfg(test)]
mod tests;

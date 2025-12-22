pub mod config;
pub mod handler;

use api::VectorDb;
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

use handler::{
    delete_point_handler, get_point_handler, health_handler, insert_point_handler, root_handler,
    search_points_handler,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<VectorDb>,
}

/// Creates the HTTP router with all VectorDB routes.
/// This can be used by both the standalone http_server binary
/// and the unified server binary.
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

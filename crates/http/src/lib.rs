pub mod handler;

use api::VectorDb;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
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
        .route("/points/batch", post(batch_insert_handler))
        .route("/points/search/batch", post(batch_search_handler))
        .with_state(app_state)
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024)) // 50MB limit
}

/// Runs the HTTP server on the specified address.
pub async fn run_http_server(db: Arc<VectorDb>, addr: SocketAddr) -> Result<(), BoxError> {
    let app = create_router(db);
    let listener = TcpListener::bind(addr).await?;
    info!("HTTP server listening on http://{}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use api::DbConfig;
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use defs::Similarity;
    use index::{IndexType, hnsw::HnswConfig, kd_tree::KDTreeConfig};
    use serde_json::json;
    use storage::StorageType;

    #[tokio::test]
    async fn in_memory_storage_http_smoke_test() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db = api::init_api(DbConfig {
            storage_type: StorageType::InMemory,
            index_type: IndexType::Flat,
            data_path: temp_dir.path().to_path_buf(),
            dimension: 3,
            similarity: Similarity::Cosine,
            hnsw_config: HnswConfig::default(),
            kd_tree_config: KDTreeConfig::default(),
        })
        .unwrap();
        let server = TestServer::new(create_router(Arc::new(db))).unwrap();

        let insert_response = server
            .post("/points")
            .json(&json!({
                "vector": [1.0, 0.0, 0.0],
                "payload": {
                    "content_type": "Text",
                    "content": "smoke-test"
                }
            }))
            .await;
        insert_response.assert_status(StatusCode::CREATED);
        let insert_body: serde_json::Value = insert_response.json();
        let point_id = insert_body["point_id"].as_str().unwrap();

        let get_response = server.get(&format!("/points/{point_id}")).await;
        get_response.assert_status_ok();
        let point_body: serde_json::Value = get_response.json();
        assert_eq!(point_body["payload"]["content"], "smoke-test");
        assert_eq!(point_body["vector"], json!([1.0, 0.0, 0.0]));

        let search_response = server
            .post("/points/search")
            .json(&json!({
                "vector": [1.0, 0.0, 0.0],
                "similarity": "Cosine",
                "limit": 1
            }))
            .await;
        search_response.assert_status_ok();
        let search_body: serde_json::Value = search_response.json();
        assert_eq!(search_body["results"], json!([point_id]));

        let delete_response = server.delete(&format!("/points/{point_id}")).await;
        delete_response.assert_status(StatusCode::NO_CONTENT);

        let missing_response = server.get(&format!("/points/{point_id}")).await;
        missing_response.assert_status(StatusCode::NOT_FOUND);
    }
}

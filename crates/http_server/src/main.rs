mod config;
mod handler;

use api::{DbConfig, VectorDb, init_api};
use axum::{
    Router,
    routing::{get, post},
};
use config::Config;
use defs::{AppError, ServerError};
use index::IndexType;
use storage::StorageType;
use tokio::net::TcpListener;
use tracing::info;

use handler::{
    delete_point_handler, get_point_handler, insert_point_handler, root_handler,
    search_points_handler,
};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: Arc<VectorDb>,
}

pub fn app(db: Arc<VectorDb>) -> Router {
    let app_state = AppState { db };
    Router::new()
        .route("/", get(root_handler))
        .route("/points", post(insert_point_handler))
        .route(
            "/points/{id}",
            get(get_point_handler).delete(delete_point_handler),
        )
        .route("/points/search", post(search_points_handler))
        .with_state(app_state)
}
#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt::init();

    let config = Config::from_env();
    info!("Loaded configuration: {:?}", config.db_path);
    info!("Vector dimension set to: {}", config.vector_dimension);

    if let Some(parent) = config.db_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create database directory");
    }

    //  db init
    let db_config = DbConfig {
        storage_type: StorageType::RocksDb,
        index_type: IndexType::Flat,
        data_path: config.db_path,
        dimension: config.vector_dimension,
    };

    let db = init_api(db_config).map_err(AppError::DbError)?;

    // axum init
    info!(" Server listening on http://{}", config.listen_addr);

    let app = app(Arc::new(db));

    let listener = TcpListener::bind(config.listen_addr)
        .await
        .map_err(|err| AppError::ServerError(ServerError::Bind(err)))?;
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|err| AppError::ServerError(ServerError::Serve(err)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::SearchResponse;
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use defs::{DenseVector, Point};
    use serde_json::json;
    use tempfile::tempdir;

    fn create_test_db() -> VectorDb {
        let temp_dir = tempdir().unwrap();
        let config = DbConfig {
            storage_type: StorageType::RocksDb,
            index_type: IndexType::Flat,
            data_path: temp_dir.path().to_path_buf(),
            dimension: 2,
        };
        init_api(config).unwrap()
    }

    fn setup_test_server() -> TestServer {
        let db = Arc::new(create_test_db());
        let test_app = app(db);
        TestServer::new(test_app).unwrap()
    }

    #[tokio::test]
    async fn test_all_routes() {
        let server = setup_test_server();
        // 1 Insert a point
        let insert_response = server
            .post("/points")
            .json(
                &json!({"vector": [0.1, 0.2], "payload": {"content_type": "Image", "content": "tester"}}),
            )
            .await;
        assert_eq!(insert_response.status_code(), StatusCode::CREATED);
        println!("Insert Test passed");

        let insert_result: handler::InsertResponse = insert_response.json();
        let point_id = insert_result.point_id;

        // 2 Get the point back
        let get_response = server.get(&format!("/points/{}", point_id)).await;
        get_response.assert_status_ok();
        let point: Point = get_response.json();

        let expected_vec: DenseVector = vec![0.1, 0.2];
        assert_eq!(point.vector.unwrap(), expected_vec);
        println!("Retrieval Test passed");

        println!("Deletion Test passed");

        // 3 Search for the point
        let search_response = server
            .post("/points/search")
            .json(&json!({
                "vector": [0.11, 0.22],
                "similarity": "Cosine",
                "limit": 1
            }))
            .await;
        search_response.assert_status_ok();
        println!("{:?}", search_response);
        let search_results: SearchResponse = search_response.json();
        assert_eq!(search_results.results.len(), 1);
        assert_eq!(search_results.results[0].to_string(), point_id.to_string());
        println!("Search Test passed");

        // 4 Delete the point
        let delete_response = server.delete(&format!("/points/{}", point_id)).await;
        assert_eq!(delete_response.status_code(), StatusCode::NO_CONTENT);

        let get_after_delete_response = server.get(&format!("/points/{}", point_id)).await;
        get_after_delete_response.assert_status_not_found();
    }
}

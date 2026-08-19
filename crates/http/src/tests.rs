use super::*;
use api::DbConfig;
use axum::http::StatusCode;
use axum_test::TestServer;
use defs::Similarity;
use index::{IndexType, hnsw::HnswConfig, kd_tree::KDTreeConfig};
use serde_json::json;
use storage::StorageType;

use defs::{ApiKeyEntry, ApiKeyRole, ApiKeyStore};

const API_KEY: &str = "full-access-key";
const READONLY_API_KEY: &str = "read-only-key";

fn test_db() -> Arc<api::VectorDb> {
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
    Arc::new(db)
}

fn test_server() -> TestServer {
    let keys = ApiKeyStore::new(vec![
        ApiKeyEntry {
            name: "full".to_string(),
            role: ApiKeyRole::ReadWrite,
            key: API_KEY.to_string(),
        },
        ApiKeyEntry {
            name: "readonly".to_string(),
            role: ApiKeyRole::ReadOnly,
            key: READONLY_API_KEY.to_string(),
        },
    ]);
    let router = create_router(test_db(), Arc::new(keys));
    TestServer::new(router).unwrap()
}

#[tokio::test]
async fn in_memory_storage_http_smoke_test() {
    let server = test_server();

    let insert_response = server
        .post("/points")
        .add_header("api-key", API_KEY)
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

    let get_response = server
        .get(&format!("/points/{point_id}"))
        .add_header("api-key", API_KEY)
        .await;
    get_response.assert_status_ok();
    let point_body: serde_json::Value = get_response.json();
    assert_eq!(point_body["payload"]["content"], "smoke-test");
    assert_eq!(point_body["vector"], json!([1.0, 0.0, 0.0]));

    let search_response = server
        .post("/points/search")
        .add_header("api-key", API_KEY)
        .json(&json!({
            "vector": [1.0, 0.0, 0.0],
            "similarity": "Cosine",
            "limit": 1
        }))
        .await;
    search_response.assert_status_ok();
    let search_body: serde_json::Value = search_response.json();
    assert_eq!(search_body["results"], json!([point_id]));

    let delete_response = server
        .delete(&format!("/points/{point_id}"))
        .add_header("api-key", API_KEY)
        .await;
    delete_response.assert_status(StatusCode::NO_CONTENT);

    let missing_response = server
        .get(&format!("/points/{point_id}"))
        .add_header("api-key", API_KEY)
        .await;
    missing_response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn public_routes_require_no_key() {
    let server = test_server();
    server.get("/").await.assert_status_ok();
    server.get("/health").await.assert_status_ok();
}

#[tokio::test]
async fn protected_routes_reject_missing_key() {
    let server = test_server();
    server
        .get("/points/00000000-0000-0000-0000-000000000000")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    server
        .post("/points")
        .json(&json!({"vector": [1.0, 0.0, 0.0], "payload": {"content_type": "Text", "content": "x"}}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_routes_reject_wrong_key() {
    let server = test_server();
    server
        .get("/points/00000000-0000-0000-0000-000000000000")
        .add_header("api-key", "not-the-right-key")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn readonly_key_can_read_but_not_write() {
    let server = test_server();

    let insert_response = server
        .post("/points")
        .add_header("api-key", API_KEY)
        .json(&json!({"vector": [1.0, 0.0, 0.0], "payload": {"content_type": "Text", "content": "x"}}))
        .await;
    let point_id = insert_response.json::<serde_json::Value>()["point_id"]
        .as_str()
        .unwrap()
        .to_string();

    server
        .get(&format!("/points/{point_id}"))
        .add_header("api-key", READONLY_API_KEY)
        .await
        .assert_status_ok();

    server
        .post("/points/search")
        .add_header("api-key", READONLY_API_KEY)
        .json(&json!({"vector": [1.0, 0.0, 0.0], "similarity": "Cosine", "limit": 1}))
        .await
        .assert_status_ok();

    server
        .delete(&format!("/points/{point_id}"))
        .add_header("api-key", READONLY_API_KEY)
        .await
        .assert_status(StatusCode::FORBIDDEN);

    server
        .post("/points")
        .add_header("api-key", READONLY_API_KEY)
        .json(&json!({"vector": [1.0, 0.0, 0.0], "payload": {"content_type": "Text", "content": "y"}}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
}

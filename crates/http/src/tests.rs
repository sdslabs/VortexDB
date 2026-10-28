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
            "dense_vector": [1.0, 0.0, 0.0],
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
    assert_eq!(point_body["dense_vector"], json!([1.0, 0.0, 0.0]));

    let search_response = server
        .post("/points/search")
        .add_header("api-key", API_KEY)
        .json(&json!({
            "limit": 1,
            "query": {"dense": {"vector": [1.0, 0.0, 0.0], "similarity": "Cosine"}}
        }))
        .await;
    search_response.assert_status_ok();
    let search_body: serde_json::Value = search_response.json();
    assert_eq!(search_body["hits"], json!([{"id": point_id, "score": 0.0}]));

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
        .json(&json!({"dense_vector": [1.0, 0.0, 0.0], "payload": {"content_type": "Text", "content": "x"}}))
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
        .json(&json!({"dense_vector": [1.0, 0.0, 0.0], "payload": {"content_type": "Text", "content": "x"}}))
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
        .json(&json!({"limit": 1, "query": {"dense": {"vector": [1.0, 0.0, 0.0], "similarity": "Cosine"}}}))
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
        .json(&json!({"dense_vector": [1.0, 0.0, 0.0], "payload": {"content_type": "Text", "content": "y"}}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn sparse_and_dense_batch_round_trip() {
    let server = test_server();
    let sparse_id = "00000000-0000-0000-0000-000000000002";

    let insert_response = server
        .post("/points/batch")
        .add_header("api-key", API_KEY)
        .json(&json!({"points": [
            {"dense_vector": [1.0, 0.0, 0.0]},
            {"id": sparse_id, "sparse_vector": {"token_ids": [1, 4], "weights": [0.5, 2.0]}}
        ]}))
        .await;
    insert_response.assert_status_ok();
    let dense_id = insert_response.json::<serde_json::Value>()["ids"][0].clone();

    let search_response = server
        .post("/points/search/batch")
        .add_header("api-key", API_KEY)
        .json(&json!({"queries": [
            {"limit": 5, "query": {"dense": {"vector": [1.0, 0.0, 0.0], "similarity": "Cosine"}}},
            {"limit": 5, "query": {"sparse": {"token_ids": [4], "weights": [1.0]}}}
        ]}))
        .await;
    search_response.assert_status_ok();
    let body: serde_json::Value = search_response.json();
    assert_eq!(body["results"][0]["hits"][0]["id"], dense_id);
    assert_eq!(
        body["results"][1]["hits"],
        json!([{"id": sparse_id, "score": 2.0}])
    );

    let point: serde_json::Value = server
        .get(&format!("/points/{sparse_id}"))
        .add_header("api-key", API_KEY)
        .await
        .json();
    assert_eq!(point["dense_vector"], json!(null));
    assert_eq!(point["sparse_vector"]["token_ids"], json!([1, 4]));
}

#[tokio::test]
async fn batch_errors_use_api_status_codes() {
    let server = test_server();
    let id = "00000000-0000-0000-0000-000000000003";
    let point = json!({"id": id, "dense_vector": [1.0, 0.0, 0.0]});

    server
        .post("/points/batch")
        .add_header("api-key", API_KEY)
        .json(&json!({"points": [point]}))
        .await
        .assert_status_ok();
    server
        .post("/points/batch")
        .add_header("api-key", API_KEY)
        .json(&json!({"points": [point]}))
        .await
        .assert_status(StatusCode::CONFLICT);
    server
        .post("/points")
        .add_header("api-key", API_KEY)
        .json(&json!({"payload": {"content_type": "Text", "content": "no vectors"}}))
        .await
        .assert_status(StatusCode::BAD_REQUEST);
}

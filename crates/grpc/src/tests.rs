use crate::constants::AUTHORIZATION_HEADER_KEY;
use crate::service::vectordb::vector_db_client::VectorDbClient;
use crate::service::vectordb::{
    ContentType, DenseQuery, DenseVector, InsertPointsBatchRequest, Payload, PointId, PointInput,
    SearchBatchRequest, SearchRequest, SparseVector, Uuid, search_request,
};
use crate::service::{VectorDBService, run_server};
use crate::utils::ServerEndpoint;
use api::DbConfig;
use defs::{ApiKeyEntry, ApiKeyRole, ApiKeyStore, Similarity};
use index::{IndexType, hnsw::HnswConfig, kd_tree::KDTreeConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use storage::StorageType;
use tempfile::{TempDir, tempdir};
use tonic::transport::Channel;

// Inspired from https://github.com/hyperium/tonic/discussions/924#discussioncomment-9854088

const TEST_AUTH_BEARER_TOKEN: &str = "123";
const TEST_READONLY_BEARER_TOKEN: &str = "456";

fn append_test_auth_header<T>(request: &mut tonic::Request<T>, token: &str) {
    let auth_value = format!("Bearer {}", token);
    request
        .metadata_mut()
        .insert(AUTHORIZATION_HEADER_KEY, auth_value.parse().unwrap());
}

async fn start_test_server() -> Result<(SocketAddr, TempDir), Box<dyn std::error::Error>> {
    // using a temporary directory for db datapath
    let temp_dir = tempdir().unwrap();

    let db_config = DbConfig {
        storage_type: StorageType::RocksDb,
        index_type: IndexType::Flat,
        data_path: temp_dir.path().to_path_buf(),
        dimension: 3,
        similarity: Similarity::Cosine,
        hnsw_config: HnswConfig::default(),
        kd_tree_config: KDTreeConfig::default(),
    };

    let vector_db_api = api::init_api(db_config)?;

    let keys = Arc::new(ApiKeyStore::new(vec![
        ApiKeyEntry {
            name: "test".to_string(),
            role: ApiKeyRole::ReadWrite,
            key: TEST_AUTH_BEARER_TOKEN.to_string(),
        },
        ApiKeyEntry {
            name: "test-readonly".to_string(),
            role: ApiKeyRole::ReadOnly,
            key: TEST_READONLY_BEARER_TOKEN.to_string(),
        },
    ]));

    let vector_db_service = VectorDBService::new(Arc::new(vector_db_api), false);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let listener_addr = listener.local_addr()?;

    tokio::spawn(async move {
        let _ = run_server(vector_db_service, ServerEndpoint::Listener(listener), keys)
            .await
            .inspect_err(|err| panic!("Could not start test server : {:?}", err));
    });

    Ok((listener_addr, temp_dir))
}

async fn create_test_client(
    server_addr: SocketAddr,
) -> Result<VectorDbClient<Channel>, Box<dyn std::error::Error>> {
    let channel = Channel::from_shared(format!("http://{}", server_addr))
        .unwrap()
        .connect()
        .await?;
    Ok(VectorDbClient::new(channel))
}

fn dense_search(values: Vec<f32>, similarity: i32, limit: u64) -> SearchRequest {
    SearchRequest {
        limit,
        query: Some(search_request::Query::Dense(DenseQuery {
            vector: Some(DenseVector { values }),
            similarity,
            ef: None,
        })),
    }
}

fn sparse_search(token_ids: Vec<u32>, weights: Vec<f32>, limit: u64) -> SearchRequest {
    SearchRequest {
        limit,
        query: Some(search_request::Query::Sparse(SparseVector {
            token_ids,
            weights,
        })),
    }
}

fn point_id(value: &str) -> PointId {
    PointId {
        id: Some(Uuid {
            value: value.to_string(),
        }),
    }
}

#[tokio::test]
async fn test_grpc_server_start() {
    let (server_addr, _temp_dir) = start_test_server().await.unwrap();
    let mut client = create_test_client(server_addr).await.unwrap();

    // insert a test vector
    let test_vec = vec![1.0, 2.0, 3.0];

    let mut request = tonic::Request::new(PointInput {
        dense_vector: Some(DenseVector {
            values: test_vec.clone(),
        }),
        payload: Some(Payload::default()),
        ..Default::default()
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);

    let status = client.insert_point(request).await.is_ok();
    assert!(status);
}

#[tokio::test]
async fn test_insert_vector_rpc() {
    let (server_addr, _temp_dir) = start_test_server().await.unwrap();
    let mut client = create_test_client(server_addr).await.unwrap();

    // insert a test vector
    let test_vec = vec![1.0, 2.0, 3.0];

    let mut request = tonic::Request::new(PointInput {
        dense_vector: Some(DenseVector {
            values: test_vec.clone(),
        }),
        payload: Some(Payload {
            content_type: ContentType::Text as i32,
            content: "test".to_string(),
        }),
        ..Default::default()
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);

    let resp = client.insert_point(request).await;

    // check if request is successful
    assert!(resp.is_ok());

    // check if the vector is actually present in the database
    let mut request = tonic::Request::new(PointId {
        id: resp.unwrap().into_inner().id,
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);
    let resp = client.get_point(request).await;

    // check if request is successful
    assert!(resp.is_ok());
    let point = resp.unwrap().into_inner();
    assert_eq!(point.dense_vector.unwrap().values, test_vec);

    // payload assertions
    let payload = point.payload.unwrap();
    assert_eq!(payload.content_type, ContentType::Text as i32);
    assert_eq!(payload.content, "test");

    // insert a new vector with mismatched dimensions
    let mut request = tonic::Request::new(PointInput {
        dense_vector: Some(DenseVector {
            values: vec![1.0, 2.0],
        }),
        payload: Some(Payload::default()),
        ..Default::default()
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);

    let resp = client.insert_point(request).await;

    // request must fail
    assert!(resp.is_err());
}

#[tokio::test]
async fn test_delete_vector_rpc() {
    let (server_addr, _temp_dir) = start_test_server().await.unwrap();
    let mut client = create_test_client(server_addr).await.unwrap();

    // insert a test vector
    let test_vec = vec![1.0, 2.0, 3.0];
    let mut request = tonic::Request::new(PointInput {
        dense_vector: Some(DenseVector {
            values: test_vec.clone(),
        }),
        payload: Some(Payload::default()),
        ..Default::default()
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);

    let resp = client.insert_point(request).await;

    // check if request is successful
    assert!(resp.is_ok());
    let point = resp.unwrap().into_inner();

    // delete the vector
    let mut request = tonic::Request::new(PointId {
        id: point.id.clone(),
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);

    let resp = client.delete_point(request).await;

    // check if request is successful
    assert!(resp.is_ok());

    // verify that the vector is deleted
    let mut request = tonic::Request::new(PointId { id: point.id });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);

    let resp = client.get_point(request).await;

    // request must fail since the vector is deleted
    assert!(resp.is_err());
}

#[tokio::test]
async fn test_search_vector_rpc() {
    let (server_addr, _temp_dir) = start_test_server().await.unwrap();
    let mut client = create_test_client(server_addr).await.unwrap();

    // insert a test vector
    let test_vec = vec![1.0, 2.0, 3.0];
    let mut request = tonic::Request::new(PointInput {
        dense_vector: Some(DenseVector {
            values: test_vec.clone(),
        }),
        payload: Some(Payload::default()),
        ..Default::default()
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);

    let resp = client.insert_point(request).await;

    // check if request is successful
    assert!(resp.is_ok());
    let point = resp.unwrap().into_inner();

    let query_vec = vec![2.0, 2.0, 2.0];

    // search for the vector
    let mut request = tonic::Request::new(dense_search(query_vec.clone(), 0, 1));
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);

    let resp = client.search(request).await;

    // check if request is successful
    assert!(resp.is_ok());
    let result = resp.unwrap().into_inner();

    // 1 vector has to be returned
    assert_eq!(result.hits.len(), 1);

    // check if the returned point id matches the inserted point id
    assert_eq!(result.hits[0].id, Some(PointId { id: point.id }));
    assert_eq!(result.hits[0].score, 2f32.sqrt());
}

#[tokio::test]
async fn test_unauthorized_rpc() {
    let (server_addr, _temp_dir) = start_test_server().await.unwrap();
    let mut client = create_test_client(server_addr).await.unwrap();

    // insert a test vector
    let test_vec = vec![1.0, 2.0, 3.0];
    let mut request = tonic::Request::new(PointInput {
        dense_vector: Some(DenseVector {
            values: test_vec.clone(),
        }),
        payload: Some(Payload::default()),
        ..Default::default()
    });

    append_test_auth_header(&mut request, "43121");
    let resp = client.insert_point(request).await;

    // request must fail
    assert!(resp.is_err());
}

#[tokio::test]
async fn test_readonly_key_cannot_write() {
    let (server_addr, _temp_dir) = start_test_server().await.unwrap();
    let mut client = create_test_client(server_addr).await.unwrap();

    let mut request = tonic::Request::new(PointInput {
        dense_vector: Some(DenseVector {
            values: vec![1.0, 2.0, 3.0],
        }),
        payload: Some(Payload::default()),
        ..Default::default()
    });
    append_test_auth_header(&mut request, TEST_READONLY_BEARER_TOKEN);

    let resp = client.insert_point(request).await;

    assert_eq!(resp.unwrap_err().code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_readonly_key_can_search() {
    let (server_addr, _temp_dir) = start_test_server().await.unwrap();
    let mut client = create_test_client(server_addr).await.unwrap();

    let mut request = tonic::Request::new(dense_search(vec![1.0, 2.0, 3.0], 0, 1));
    append_test_auth_header(&mut request, TEST_READONLY_BEARER_TOKEN);

    let resp = client.search(request).await;

    assert!(resp.is_ok());
}

#[tokio::test]
async fn test_sparse_and_mixed_batch_rpc() {
    let (server_addr, _temp_dir) = start_test_server().await.unwrap();
    let mut client = create_test_client(server_addr).await.unwrap();
    let sparse_id = "00000000-0000-0000-0000-000000000002";

    let mut request = tonic::Request::new(InsertPointsBatchRequest {
        points: vec![
            PointInput {
                dense_vector: Some(DenseVector {
                    values: vec![1.0, 0.0, 0.0],
                }),
                ..Default::default()
            },
            PointInput {
                id: Some(point_id(sparse_id)),
                sparse_vector: Some(SparseVector {
                    token_ids: vec![1, 4],
                    weights: vec![0.5, 2.0],
                }),
                ..Default::default()
            },
        ],
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);
    let ids = client
        .insert_points_batch(request)
        .await
        .unwrap()
        .into_inner()
        .ids;
    assert_eq!(ids[1], point_id(sparse_id));

    let mut request = tonic::Request::new(point_id(sparse_id));
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);
    let point = client.get_point(request).await.unwrap().into_inner();
    assert!(point.dense_vector.is_none());
    assert_eq!(point.sparse_vector.unwrap().token_ids, vec![1, 4]);

    let mut request = tonic::Request::new(SearchBatchRequest {
        queries: vec![
            dense_search(vec![1.0, 0.0, 0.0], 3, 5),
            sparse_search(vec![4], vec![1.0], 5),
        ],
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);
    let results = client
        .search_batch(request)
        .await
        .unwrap()
        .into_inner()
        .results;
    assert_eq!(results[0].hits[0].id, Some(ids[0].clone()));
    assert_eq!(results[1].hits.len(), 1);
    assert_eq!(results[1].hits[0].id, Some(point_id(sparse_id)));
    assert_eq!(results[1].hits[0].score, 2.0);
}

#[tokio::test]
async fn test_invalid_requests_return_invalid_argument() {
    let (server_addr, _temp_dir) = start_test_server().await.unwrap();
    let mut client = create_test_client(server_addr).await.unwrap();

    let mut request = tonic::Request::new(point_id("not-a-uuid"));
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);
    let status = client.get_point(request).await.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);

    let mut request = tonic::Request::new(SearchRequest {
        limit: 1,
        query: None,
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);
    let status = client.search(request).await.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);

    let mut request = tonic::Request::new(PointInput {
        payload: Some(Payload::default()),
        ..Default::default()
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);
    let status = client.insert_point(request).await.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}

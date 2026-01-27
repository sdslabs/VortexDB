use crate::constants::AUTHORIZATION_HEADER_KEY;
use crate::service::vectordb::vector_db_client::VectorDbClient;
use crate::service::vectordb::{
    ContentType, DenseVector, InsertVectorRequest, Payload, PointId, SearchRequest,
};
use crate::service::{VectorDBService, run_server};
use crate::utils::ServerEndpoint;
use api::DbConfig;
use defs::Similarity;
use index::IndexType;
use std::net::SocketAddr;
use std::sync::Arc;
use storage::StorageType;
use tempfile::{TempDir, tempdir};
use tonic::transport::Channel;

// Inspired from https://github.com/hyperium/tonic/discussions/924#discussioncomment-9854088

const TEST_AUTH_BEARER_TOKEN: &str = "123";

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
    };

    let vector_db_api = api::init_api(db_config)?;

    let vector_db_service = VectorDBService::new(Arc::new(vector_db_api), false);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let listener_addr = listener.local_addr()?;

    tokio::spawn(async move {
        let _ = run_server(
            vector_db_service,
            ServerEndpoint::Listener(listener),
            TEST_AUTH_BEARER_TOKEN.to_string(),
        )
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

#[tokio::test]
async fn test_grpc_server_start() {
    let (server_addr, _temp_dir) = start_test_server().await.unwrap();
    let mut client = create_test_client(server_addr).await.unwrap();

    // insert a test vector
    let test_vec = vec![1.0, 2.0, 3.0];

    let mut request = tonic::Request::new(InsertVectorRequest {
        vector: Some(DenseVector {
            values: test_vec.clone(),
        }),
        payload: Some(Payload::default()),
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);

    let status = client.insert_vector(request).await.is_ok();
    assert!(status);
}

#[tokio::test]
async fn test_insert_vector_rpc() {
    let (server_addr, _temp_dir) = start_test_server().await.unwrap();
    let mut client = create_test_client(server_addr).await.unwrap();

    // insert a test vector
    let test_vec = vec![1.0, 2.0, 3.0];

    let mut request = tonic::Request::new(InsertVectorRequest {
        vector: Some(DenseVector {
            values: test_vec.clone(),
        }),
        payload: Some(Payload {
            content_type: ContentType::Text as i32,
            content: "test".to_string(),
        }),
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);

    let resp = client.insert_vector(request).await;

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
    assert_eq!(point.vector.unwrap().values, test_vec);

    // payload assertions
    let payload = point.payload.unwrap();
    assert_eq!(payload.content_type, ContentType::Text as i32);
    assert_eq!(payload.content, "test");

    // insert a new vector with mismatched dimensions
    let mut request = tonic::Request::new(InsertVectorRequest {
        vector: Some(DenseVector {
            values: vec![1.0, 2.0],
        }),
        payload: Some(Payload::default()),
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);

    let resp = client.insert_vector(request).await;

    // request must fail
    assert!(resp.is_err());
}

#[tokio::test]
async fn test_delete_vector_rpc() {
    let (server_addr, _temp_dir) = start_test_server().await.unwrap();
    let mut client = create_test_client(server_addr).await.unwrap();

    // insert a test vector
    let test_vec = vec![1.0, 2.0, 3.0];
    let mut request = tonic::Request::new(InsertVectorRequest {
        vector: Some(DenseVector {
            values: test_vec.clone(),
        }),
        payload: Some(Payload::default()),
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);

    let resp = client.insert_vector(request).await;

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
    let mut request = tonic::Request::new(InsertVectorRequest {
        vector: Some(DenseVector {
            values: test_vec.clone(),
        }),
        payload: Some(Payload::default()),
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);

    let resp = client.insert_vector(request).await;

    // check if request is successful
    assert!(resp.is_ok());
    let point = resp.unwrap().into_inner();

    let query_vec = vec![2.0, 2.0, 2.0];

    // search for the vector
    let mut request = tonic::Request::new(SearchRequest {
        query_vector: Some(DenseVector {
            values: query_vec.clone(),
        }),
        similarity: 0, // euclidean distance
        limit: 1,
    });
    append_test_auth_header(&mut request, TEST_AUTH_BEARER_TOKEN);

    let resp = client.search_points(request).await;

    // check if request is successful
    assert!(resp.is_ok());
    let result = resp.unwrap().into_inner();

    // 1 vector has to be returned
    assert_eq!(result.result_point_ids.len(), 1);

    // check if the returned point id matches the inserted point id
    assert_eq!(result.result_point_ids[0], PointId { id: point.id });
}

#[tokio::test]
async fn test_unauthorized_rpc() {
    let (server_addr, _temp_dir) = start_test_server().await.unwrap();
    let mut client = create_test_client(server_addr).await.unwrap();

    // insert a test vector
    let test_vec = vec![1.0, 2.0, 3.0];
    let mut request = tonic::Request::new(InsertVectorRequest {
        vector: Some(DenseVector {
            values: test_vec.clone(),
        }),
        payload: Some(Payload::default()),
    });

    append_test_auth_header(&mut request, "43121");
    let resp = client.insert_vector(request).await;

    // request must fail
    assert!(resp.is_err());
}

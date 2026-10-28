use std::str::FromStr;
use std::sync::Arc;

use crate::error::GrpcError;
use crate::interceptors;
use crate::service::vectordb::{ContentType, Uuid};
use crate::utils::log_rpc;
use crate::{constants::SIMILARITY_PROTOBUFF_MAP, utils::ServerEndpoint};
use tonic::{Request, Response, Status, service::InterceptorLayer, transport::Server};
use tracing::{Level, event};
use uuid::Uuid as UuidCrate;
use vectordb::{
    DenseVector, InsertPointsBatchRequest, InsertPointsBatchResponse, Point, PointId, PointInput,
    SearchBatchRequest, SearchBatchResponse, SearchHit, SearchRequest, SearchResponse,
    SparseVector, search_request,
    vector_db_server::{VectorDb, VectorDbServer},
};

pub mod vectordb {
    tonic::include_proto!("vectordb");
}

pub struct VectorDBService {
    pub vector_db: Arc<api::VectorDb>,
    pub logging: bool,
}

impl VectorDBService {
    pub fn new(vector_db: Arc<api::VectorDb>, logging: bool) -> Self {
        Self { vector_db, logging }
    }
}

fn parse_point_id(point_id: PointId) -> Result<UuidCrate, Status> {
    let value = point_id.id.unwrap_or_default().value;
    UuidCrate::from_str(&value)
        .map_err(|_| Status::invalid_argument(format!("invalid point id: {}", value)))
}

fn to_proto_point_id(id: UuidCrate) -> PointId {
    PointId {
        id: Some(Uuid {
            value: id.to_string(),
        }),
    }
}

fn from_proto_payload(payload: vectordb::Payload) -> defs::Payload {
    let content_type =
        match ContentType::try_from(payload.content_type).unwrap_or(ContentType::Text) {
            ContentType::Text => defs::ContentType::Text,
            ContentType::Image => defs::ContentType::Image,
        };
    defs::Payload {
        content_type,
        content: payload.content,
    }
}

fn to_proto_payload(payload: defs::Payload) -> vectordb::Payload {
    let content_type = match payload.content_type {
        defs::ContentType::Text => ContentType::Text,
        defs::ContentType::Image => ContentType::Image,
    };
    vectordb::Payload {
        content_type: content_type as i32,
        content: payload.content,
    }
}

fn from_proto_sparse(sparse_vector: SparseVector) -> defs::SparseVector {
    defs::SparseVector {
        token_ids: sparse_vector.token_ids,
        weights: sparse_vector.weights,
    }
}

fn from_proto_point_input(point: PointInput) -> Result<defs::PointInput, Status> {
    Ok(defs::PointInput {
        id: point.id.map(parse_point_id).transpose()?,
        dense_vector: point.dense_vector.map(|v| v.values),
        sparse_vector: point.sparse_vector.map(from_proto_sparse),
        payload: point.payload.map(from_proto_payload),
    })
}

fn to_proto_point(point: defs::Point) -> Point {
    Point {
        id: Some(to_proto_point_id(point.id)),
        dense_vector: point.dense_vector.map(|values| DenseVector { values }),
        sparse_vector: point.sparse_vector.map(|sv| SparseVector {
            token_ids: sv.token_ids,
            weights: sv.weights,
        }),
        payload: point.payload.map(to_proto_payload),
    }
}

fn from_proto_search_request(request: SearchRequest) -> Result<defs::SearchQuery, Status> {
    let query = match request
        .query
        .ok_or_else(|| Status::invalid_argument("query is required"))?
    {
        search_request::Query::Dense(dense) => {
            let similarity = SIMILARITY_PROTOBUFF_MAP
                .get(dense.similarity as usize)
                .ok_or_else(|| Status::invalid_argument("Invalid similarity"))?;
            let vector = dense
                .vector
                .ok_or_else(|| Status::invalid_argument("dense query vector is required"))?;
            defs::QueryVector::Dense(defs::DenseQuery {
                vector: vector.values,
                similarity: *similarity,
                ef: dense.ef.map(|ef| ef as usize),
            })
        }
        search_request::Query::Sparse(sparse_vector) => {
            defs::QueryVector::Sparse(from_proto_sparse(sparse_vector))
        }
    };

    Ok(defs::SearchQuery {
        limit: request.limit as usize,
        query,
    })
}

fn to_proto_search_response(hits: Vec<defs::SearchHit>) -> SearchResponse {
    SearchResponse {
        hits: hits
            .into_iter()
            .map(|hit| SearchHit {
                id: Some(to_proto_point_id(hit.id)),
                score: hit.score,
            })
            .collect(),
    }
}

#[tonic::async_trait]
impl VectorDb for VectorDBService {
    async fn insert_point(
        &self,
        request: Request<PointInput>,
    ) -> Result<Response<PointId>, Status> {
        log_rpc("insert_point", self.logging);
        interceptors::require_write_role(&request)?;

        let point = from_proto_point_input(request.into_inner())?;
        let id = self
            .vector_db
            .insert(point)
            .map_err(|e| Status::from(GrpcError::from(e)))?;

        Ok(Response::new(to_proto_point_id(id)))
    }

    async fn get_point(&self, request: Request<PointId>) -> Result<Response<Point>, Status> {
        log_rpc("get_point", self.logging);

        let point_id = parse_point_id(request.into_inner())?;
        let point = self
            .vector_db
            .get(point_id)
            .map_err(|e| Status::from(GrpcError::from(e)))?
            .ok_or_else(|| Status::not_found(format!("point not found: {}", point_id)))?;

        Ok(Response::new(to_proto_point(point)))
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        log_rpc("search", self.logging);

        let query = from_proto_search_request(request.into_inner())?;
        let hits = self
            .vector_db
            .search(query)
            .map_err(|e| Status::from(GrpcError::from(e)))?;

        Ok(Response::new(to_proto_search_response(hits)))
    }

    async fn delete_point(&self, request: Request<PointId>) -> Result<Response<()>, Status> {
        log_rpc("delete_point", self.logging);
        interceptors::require_write_role(&request)?;

        let point_id = parse_point_id(request.into_inner())?;

        match self.vector_db.delete(point_id) {
            Ok(true) => Ok(Response::new(())),
            Ok(false) => Err(Status::not_found("Point not found")),
            Err(e) => Err(Status::from(GrpcError::from(e))),
        }
    }

    async fn insert_points_batch(
        &self,
        request: Request<InsertPointsBatchRequest>,
    ) -> Result<Response<InsertPointsBatchResponse>, Status> {
        interceptors::require_write_role(&request)?;

        let points = request
            .into_inner()
            .points
            .into_iter()
            .map(from_proto_point_input)
            .collect::<Result<Vec<_>, Status>>()?;

        let ids = self
            .vector_db
            .insert_batch(points)
            .map_err(|e| Status::from(GrpcError::from(e)))?;

        Ok(Response::new(InsertPointsBatchResponse {
            ids: ids.into_iter().map(to_proto_point_id).collect(),
        }))
    }

    async fn search_batch(
        &self,
        request: Request<SearchBatchRequest>,
    ) -> Result<Response<SearchBatchResponse>, Status> {
        let queries = request
            .into_inner()
            .queries
            .into_iter()
            .map(from_proto_search_request)
            .collect::<Result<Vec<_>, Status>>()?;

        let results = self
            .vector_db
            .search_batch(queries)
            .map_err(|e| Status::from(GrpcError::from(e)))?
            .into_iter()
            .map(to_proto_search_response)
            .collect();

        Ok(Response::new(SearchBatchResponse { results }))
    }
}
pub async fn run_server(
    vector_db_service: VectorDBService,
    endpoint: ServerEndpoint,
    keys: Arc<defs::ApiKeyStore>,
) -> Result<(), Box<dyn std::error::Error>> {
    event!(Level::INFO, "Starting gRPC server at: {:?}", endpoint);

    let auth_interceptor = interceptors::AuthInterceptor::new(keys);

    let router = Server::builder()
        .layer(InterceptorLayer::new(auth_interceptor))
        .add_service(VectorDbServer::new(vector_db_service));

    match endpoint {
        ServerEndpoint::Address(addr) => {
            router.serve(addr).await.map_err(|err| {
                event!(
                    Level::ERROR,
                    "Failed to start gRPC server with address: {:?}",
                    err
                );
                Status::internal(format!("Failed to start server with address: {}", err))
            })?;
        }
        ServerEndpoint::Listener(listener) => {
            router
                .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
                .await
                .map_err(|err| {
                    event!(
                        Level::ERROR,
                        "Failed to start gRPC server with listener: {:?}",
                        err
                    );
                    Status::internal(format!("Failed to start server with listener: {}", err))
                })?;
        }
    }
    Ok(())
}

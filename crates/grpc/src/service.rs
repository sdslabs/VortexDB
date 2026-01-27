use std::str::FromStr;
use std::sync::Arc;

use crate::interceptors;
use crate::service::vectordb::{ContentType, Uuid};
use crate::utils::log_rpc;
use crate::{constants::SIMILARITY_PROTOBUFF_MAP, utils::ServerEndpoint};
use tonic::{Request, Response, Status, service::InterceptorLayer, transport::Server};
use tracing::{Level, event};
use uuid::Uuid as UuidCrate;
use vectordb::{
    DenseVector, InsertVectorRequest, Point, PointId, SearchRequest, SearchResponse,
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

#[tonic::async_trait]
impl VectorDb for VectorDBService {
    async fn insert_vector(
        &self,
        request: Request<InsertVectorRequest>,
    ) -> Result<Response<PointId>, Status> {
        log_rpc("insert_vector", self.logging);

        let inner_request = request.into_inner();

        let dense_vector = inner_request.vector;
        if dense_vector.is_none() {
            return Err(Status::invalid_argument("dense_vector is empty"));
        }

        // TODO: Implement payload handling once its defined
        // fetch payload and default to empty struct otherwise

        let payload = inner_request.payload.unwrap_or_default();
        let payload_content = payload.content;
        let payload_type =
            match ContentType::try_from(payload.content_type).unwrap_or(ContentType::Text) {
                ContentType::Text => defs::ContentType::Text,
                ContentType::Image => defs::ContentType::Image,
            };

        let point_id = self.vector_db.insert(
            dense_vector.unwrap().values,
            defs::Payload {
                content_type: payload_type,
                content: payload_content,
            },
        );

        let res =
            point_id.map_err(|e| Status::internal(format!("failed to insert vector: {:?}", e)))?;

        Ok(Response::new(PointId {
            id: Some(Uuid {
                value: res.to_string(),
            }),
        }))
    }

    async fn get_point(&self, request: Request<PointId>) -> Result<Response<Point>, Status> {
        log_rpc("get_point", self.logging);

        let inner_request = request.into_inner();

        let point_id = inner_request.id.unwrap_or_default().value;
        let point_opt = self
            .vector_db
            .get(UuidCrate::from_str(&point_id).unwrap())
            .map_err(|e| Status::aborted(format!("point not found {:?}", e)))?;

        // return error if not found
        let point = point_opt.ok_or(Status::not_found(format!("point not found: {}", point_id)))?;

        let payload = point.payload.map(|p| vectordb::Payload {
            content_type: match p.content_type {
                defs::ContentType::Text => ContentType::Text as i32,
                defs::ContentType::Image => ContentType::Image as i32,
            },
            content: p.content,
        });

        Ok(Response::new(Point {
            id: Some(PointId {
                id: Some(Uuid {
                    value: point.id.to_string(),
                }),
            }),
            vector: Some(DenseVector {
                values: point.vector.unwrap_or_default(),
            }),
            payload,
        }))
    }

    async fn search_points(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        log_rpc("search_points", self.logging);

        let search_request = request.into_inner();

        // extract request arguments
        let query_vect = search_request
            .query_vector
            .ok_or(Status::invalid_argument("Invalid query_vector"))?;
        let similarity = SIMILARITY_PROTOBUFF_MAP
            .get(search_request.similarity as usize)
            .ok_or(Status::invalid_argument("Invalid similarity"))?;
        let limit = search_request.limit;

        if limit == 0 {
            return Err(Status::invalid_argument("Limit must be greater than zero"));
        }

        let result_point_ids = self
            .vector_db
            .search(query_vect.values, *similarity, limit as usize)
            .map_err(|_| Status::internal("Internal server error"))?;

        // create a mapped vector of PointIds
        let result = result_point_ids
            .into_iter()
            .map(|id| PointId {
                id: Some(Uuid {
                    value: id.to_string(),
                }),
            })
            .collect();

        Ok(Response::new(SearchResponse {
            result_point_ids: result,
        }))
    }

    async fn delete_point(&self, request: Request<PointId>) -> Result<Response<()>, Status> {
        log_rpc("delete_point", self.logging);

        let point_id = request.into_inner().id.unwrap_or_default().value;

        match self
            .vector_db
            .delete(UuidCrate::from_str(&point_id).unwrap())
        {
            Ok(found) => {
                if found {
                    Ok(Response::new(()))
                } else {
                    Err(Status::not_found("Point not found"))
                }
            }
            Err(_) => Err(Status::internal("Error deleting point")),
        }
    }
}

pub async fn run_server(
    vector_db_service: VectorDBService,
    endpoint: ServerEndpoint,
    root_password: String,
) -> Result<(), Box<dyn std::error::Error>> {
    event!(Level::INFO, "Starting gRPC server at: {:?}", endpoint);

    let auth_interceptor = interceptors::AuthInterceptor::new(root_password);

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

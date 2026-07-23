use api::error::ApiError;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use defs::{
    AppError, BatchInsertRequest, BatchInsertResponse, BatchSearchRequest, BatchSearchResponse,
    DenseVector, Payload, Point, PointId, SearchQueryInput, SearchResponse,
};
use index::error::IndexError;
use serde::{Deserialize, Serialize};
use storage::error::StorageError;
use tracing::error;

use crate::AppState;

#[derive(Deserialize, Debug)]
pub struct InsertRequest {
    pub vector: DenseVector,
    pub payload: Payload,
}

#[derive(Serialize, Deserialize)]
pub struct InsertResponse {
    pub point_id: PointId,
}

pub async fn root_handler() -> &'static str {
    "Vector Database server is running!"
}

pub async fn health_handler() -> &'static str {
    "OK"
}

pub async fn insert_point_handler(
    State(app_state): State<AppState>,
    Json(request): Json<InsertRequest>,
) -> Result<(StatusCode, Json<InsertResponse>), (StatusCode, String)> {
    match app_state.db.insert(request.vector, request.payload) {
        Ok(point_id) => {
            let response = InsertResponse { point_id };
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => {
            error!("Failed to insert point: {:?}", e);
            Err(api_error_to_response(&e))
        }
    }
}

pub async fn batch_insert_handler(
    State(state): State<AppState>,
    Json(request): Json<BatchInsertRequest>,
) -> Result<Json<BatchInsertResponse>, AppError> {
    let ids = state
        .db
        .insert_batch(request.points)
        .map_err(|e| AppError::Api(e.to_string()))?;

    Ok(Json(BatchInsertResponse {
        inserted: ids.len(),
        ids,
    }))
}

pub async fn get_point_handler(
    Path(point_id): Path<PointId>,
    State(app_state): State<AppState>,
) -> Result<Json<Point>, (StatusCode, String)> {
    match app_state.db.get(point_id) {
        Ok(Some(point)) => Ok(Json(point)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Point not found".to_string())),
        Err(e) => {
            error!("Failed to get point {}: {:?}", point_id, e);
            Err(api_error_to_response(&e))
        }
    }
}

pub async fn delete_point_handler(
    Path(point_id): Path<PointId>,
    State(app_state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    match app_state.db.delete(point_id) {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete point {}: {:?}", point_id, e);
            Err(api_error_to_response(&e))
        }
    }
}

pub async fn search_points_handler(
    State(app_state): State<AppState>,
    Json(request): Json<SearchQueryInput>,
) -> Result<Json<SearchResponse>, (StatusCode, String)> {
    match app_state.db.search(request) {
        Ok(results) => {
            let response = SearchResponse { results };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to search points: {:?}", e);
            Err(api_error_to_response(&e))
        }
    }
}

pub async fn batch_search_handler(
    State(state): State<AppState>,
    Json(request): Json<BatchSearchRequest>,
) -> Result<Json<BatchSearchResponse>, AppError> {
    let results = state
        .db
        .search_batch(request.queries)
        .map_err(|e| AppError::Api(e.to_string()))?
        .into_iter()
        .map(|ids| SearchResponse { results: ids })
        .collect();

    Ok(Json(BatchSearchResponse { results }))
}

/// Map `ApiError` into an HTTP `(StatusCode, String)` response.
fn api_error_to_response(err: &ApiError) -> (StatusCode, String) {
    match err {
        ApiError::PointNotFound { .. } => (StatusCode::NOT_FOUND, err.to_string()),
        ApiError::DimensionMismatch { .. } => (StatusCode::BAD_REQUEST, err.to_string()),
        ApiError::InvalidSearchLimit { .. } => (StatusCode::BAD_REQUEST, err.to_string()),
        ApiError::LockError => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Server is busy, try again".to_string(),
        ),
        ApiError::InitializationFailed { .. } => {
            (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
        }
        ApiError::Storage { source } => match source {
            // storage errors are internal IO/serialization issues
            StorageError::RocksDbOpen { .. }
            | StorageError::RocksDbRead { .. }
            | StorageError::RocksDbWrite { .. }
            | StorageError::RocksDbDelete { .. }
            | StorageError::RocksDbIteration { .. }
            | StorageError::Serialization { .. }
            | StorageError::Deserialization { .. }
            | StorageError::RocksDbCheckpoint { .. }
            | StorageError::RocksDbFlush { .. }
            | StorageError::RocksDbInitialization { .. }
            | StorageError::RocksDbCheckpointMsg { .. }
            | StorageError::RocksDbCheckpointIo { .. }
            | StorageError::InMemoryLock { .. }
            | StorageError::InMemoryCheckpoint { .. }
            | StorageError::InMemoryCheckpointIo { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, source.to_string())
            }
        },
        ApiError::Index { source } => match source {
            IndexError::PointNotFound { .. } => (StatusCode::NOT_FOUND, source.to_string()),
            IndexError::PointAlreadyExists { .. } => (StatusCode::CONFLICT, source.to_string()),
            IndexError::InvalidSearchLimit { .. } => (StatusCode::BAD_REQUEST, source.to_string()),
            IndexError::DimensionMismatch { .. } => (StatusCode::BAD_REQUEST, source.to_string()),
            IndexError::UnsupportedSimilarity { .. } => {
                (StatusCode::BAD_REQUEST, source.to_string())
            }
            IndexError::InvalidParameter { .. } => (StatusCode::BAD_REQUEST, source.to_string()),
            IndexError::NotInitialized | IndexError::EmptyIndex => {
                (StatusCode::FAILED_DEPENDENCY, source.to_string())
            }
            IndexError::HnswError { .. } | IndexError::SearchFailed { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, source.to_string())
            }
        },
    }
}

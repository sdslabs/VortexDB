use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use defs::{DenseVector, Payload, Point, PointId, Similarity};
use serde::{Deserialize, Serialize};
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
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to insert point".to_string(),
            ))
        }
    }
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
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ))
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
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ))
        }
    }
}

#[derive(Deserialize)]
pub struct SearchRequest {
    pub vector: DenseVector,
    pub similarity: Similarity,
    pub limit: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResponse {
    pub results: Vec<PointId>,
}

pub async fn search_points_handler(
    State(app_state): State<AppState>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, String)> {
    match app_state
        .db
        .search(request.vector, request.similarity, request.limit)
    {
        Ok(results) => {
            let response = SearchResponse { results };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to search points: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error during search".to_string(),
            ))
        }
    }
}

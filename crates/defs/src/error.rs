use std::io;

use crate::{Dimension, PointId};
#[derive(Debug, PartialEq, Eq)]
pub enum DbError {
    ParseError,
    StorageError(String),
    SerializationError(String),
    DeserializationError,
    IndexError(String),
    LockError,
    IndexInitError, //TODO: Change this
    UnsupportedSimilarity,
    DimensionMismatch,
    SnapshotError(String),
    StorageInitializationError,
    StorageCheckpointError(String),
    InvalidMagicBytes(String),
    VectorNotFound(uuid::Uuid),
    SnapshotRegistryError(String),
    SnapshotEngineError(String),
    InvalidDimension { expected: Dimension, got: Dimension },
    PointAlreadyExists { id: PointId },
    PointNotFound { id: PointId },
}

use axum::{http::StatusCode, response::IntoResponse};
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum ServerError {
    #[snafu(display("Failed to bind: {source}"))]
    Bind { source: io::Error },

    #[snafu(display("Failed to serve: {source}"))]
    Serve { source: io::Error },
}

#[derive(Debug)]
pub enum AppError {
    ServerError(ServerError),
    Api(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Api(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::ServerError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        (status, message).into_response()
    }
}
// Error type for server
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

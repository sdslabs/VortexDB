use defs::{Dimension, PointId, SparseError};
use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ApiError {
    #[snafu(display("Vector dimension mismatch: expected {expected}, got {got}"))]
    DimensionMismatch { expected: Dimension, got: Dimension },

    #[snafu(display("Failed to acquire lock on vector index"))]
    LockError,

    #[snafu(display("Storage error: {source}"))]
    Storage {
        source: storage::error::StorageError,
    },

    #[snafu(display("Index error: {source}"))]
    Index { source: index::error::IndexError },

    #[snafu(display("Point {id} not found"))]
    PointNotFound { id: PointId },

    #[snafu(display("Point {id} has neither a dense nor a sparse vector"))]
    MissingVector { id: PointId },

    #[snafu(display("Point {id} already exists"))]
    PointAlreadyExists { id: PointId },

    #[snafu(display("Invalid sparse vector: {source}"))]
    InvalidSparseVector { source: SparseError },

    #[snafu(display("Invalid search limit: {limit}"))]
    InvalidSearchLimit { limit: usize },

    #[snafu(display("Failed to initialize database: {reason}"))]
    InitializationFailed { reason: String },
}

pub type Result<T, E = ApiError> = std::result::Result<T, E>;

// Automatic conversion from StorageError to ApiError
impl From<storage::error::StorageError> for ApiError {
    fn from(source: storage::error::StorageError) -> Self {
        ApiError::Storage { source }
    }
}

// Automatic conversion from IndexError to ApiError
impl From<index::error::IndexError> for ApiError {
    fn from(source: index::error::IndexError) -> Self {
        ApiError::Index { source }
    }
}

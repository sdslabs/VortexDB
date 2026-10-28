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

use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum ServerError {
    #[snafu(display("Failed to bind: {source}"))]
    Bind { source: io::Error },

    #[snafu(display("Failed to serve: {source}"))]
    Serve { source: io::Error },
}

// Error type for server
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Snafu)]
pub enum SparseError {
    #[snafu(display("token_ids length {ids} does not match weights length {weights}"))]
    LengthMismatch { ids: usize, weights: usize },

    #[snafu(display("sparse vector must not be empty"))]
    Empty,

    #[snafu(display("token_ids must be strictly increasing, violated at index {index}"))]
    UnsortedTokenIds { index: usize },

    #[snafu(display("weight at index {index} is not finite"))]
    NonFiniteWeight { index: usize },

    #[snafu(display("weight at index {index} is negative"))]
    NegativeWeight { index: usize },
}

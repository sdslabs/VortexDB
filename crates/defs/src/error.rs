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
    InvalidDimension { expected: Dimension, got: Dimension },
    PointAlreadyExists { id: PointId },
    PointNotFound { id: PointId },
}

#[derive(Debug)]
pub enum ServerError {
    Bind(io::Error),
    Serve(io::Error),
}

#[derive(Debug)]
pub enum AppError {
    DbError(DbError),
    ServerError(ServerError),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for DbError {}

// Error type for server
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

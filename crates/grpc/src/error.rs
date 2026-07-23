use snafu::prelude::*;
use tonic::Status;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum GrpcError {
    #[snafu(display("Missing required environment variable: {var}"))]
    MissingRequiredEnvVar { var: String },

    #[snafu(display("Invalid dimension value: expected {expected}, got {got}"))]
    InvalidDimension { expected: usize, got: usize },

    #[snafu(display("Invalid data path: {source}"))]
    InvalidDataPath { source: std::io::Error },

    #[snafu(display("IO error: {source}"))]
    Io { source: std::io::Error },

    #[snafu(display("Not found: {message}"))]
    NotFound { message: String },

    #[snafu(display("Invalid argument: {message}"))]
    InvalidArgument { message: String },

    #[snafu(display("Already exists: {message}"))]
    AlreadyExists { message: String },

    #[snafu(display("Failed precondition: {message}"))]
    FailedPrecondition { message: String },

    #[snafu(display("Internal error: {message}"))]
    Internal { message: String },
}

pub type Result<T, E = GrpcError> = std::result::Result<T, E>;

impl From<api::error::ApiError> for GrpcError {
    fn from(e: api::error::ApiError) -> Self {
        use api::error::ApiError;
        match e {
            ApiError::DimensionMismatch { expected, got } => GrpcError::InvalidArgument {
                message: format!("dimension mismatch: expected {}, got {}", expected, got),
            },
            ApiError::LockError => GrpcError::Internal {
                message: "failed to acquire lock on index".to_string(),
            },
            ApiError::Storage { source } => GrpcError::Internal {
                message: format!("storage error: {:?}", source),
            },
            ApiError::Index { source } => GrpcError::Internal {
                message: format!("index error: {:?}", source),
            },
            ApiError::PointNotFound { id } => GrpcError::NotFound {
                message: format!("point not found: {}", id),
            },
            ApiError::InvalidSearchLimit { limit } => GrpcError::InvalidArgument {
                message: format!("invalid search limit: {}", limit),
            },
            ApiError::InitializationFailed { reason } => GrpcError::Internal { message: reason },
        }
    }
}

impl From<index::error::IndexError> for GrpcError {
    fn from(e: index::error::IndexError) -> Self {
        use index::error::IndexError;
        match e {
            IndexError::DimensionMismatch { expected, got } => GrpcError::InvalidArgument {
                message: format!("dimension mismatch: expected {}, got {}", expected, got),
            },
            IndexError::NotInitialized => GrpcError::FailedPrecondition {
                message: "index not initialized".to_string(),
            },
            IndexError::UnsupportedSimilarity { metric } => GrpcError::InvalidArgument {
                message: format!("unsupported similarity metric: {}", metric),
            },
            IndexError::PointNotFound { id } => GrpcError::NotFound {
                message: format!("point not found: {}", id),
            },
            IndexError::PointAlreadyExists { id } => GrpcError::AlreadyExists {
                message: format!("point already exists: {}", id),
            },
            IndexError::HnswError { message } => GrpcError::Internal { message },
            IndexError::InvalidParameter { parameter, reason } => GrpcError::InvalidArgument {
                message: format!("invalid parameter {}: {}", parameter, reason),
            },
            IndexError::SearchFailed { reason } => GrpcError::Internal { message: reason },
            IndexError::EmptyIndex => GrpcError::FailedPrecondition {
                message: "index is empty".to_string(),
            },
            IndexError::InvalidSearchLimit { limit } => GrpcError::InvalidArgument {
                message: format!("invalid search limit: {}", limit),
            },
        }
    }
}

impl From<storage::error::StorageError> for GrpcError {
    fn from(e: storage::error::StorageError) -> Self {
        use storage::error::StorageError;
        match e {
            StorageError::RocksDbOpen { path, source: _ } => GrpcError::Internal {
                message: format!("failed to open storage at {}", path),
            },
            StorageError::RocksDbRead { id, source: _ } => GrpcError::Internal {
                message: format!("failed to read point {} from storage", id),
            },
            StorageError::RocksDbWrite { id, source: _ } => GrpcError::Internal {
                message: format!("failed to write point {} to storage", id),
            },
            StorageError::RocksDbDelete { id, source: _ } => GrpcError::Internal {
                message: format!("failed to delete point {} from storage", id),
            },
            StorageError::RocksDbIteration { source: _ } => GrpcError::Internal {
                message: "failed to iterate over storage".to_string(),
            },
            StorageError::Serialization { id, source: _ } => GrpcError::Internal {
                message: format!("failed to serialize point {}", id),
            },
            StorageError::Deserialization { id, source: _ } => GrpcError::Internal {
                message: format!("failed to deserialize point {}", id),
            },
            StorageError::RocksDbInitialization {} => GrpcError::Internal {
                message: "failed to initialize storage".to_string(),
            },
            StorageError::RocksDbCheckpointMsg { msg } => GrpcError::Internal {
                message: format!("checkpoint error: {}", msg),
            },
            StorageError::RocksDbCheckpointIo { msg, source: _ } => GrpcError::Internal {
                message: format!("checkpoint io error: {}", msg),
            },
            StorageError::RocksDbCheckpoint { source: _ } => GrpcError::Internal {
                message: "checkpoint error".to_string(),
            },
            StorageError::RocksDbFlush { source: _ } => GrpcError::Internal {
                message: "flush error".to_string(),
            },
            StorageError::InMemoryLock {} => GrpcError::Internal {
                message: "failed to lock in-memory storage".to_string(),
            },
            StorageError::InMemoryCheckpoint { msg } => GrpcError::Internal {
                message: format!("in-memory checkpoint error: {}", msg),
            },
            StorageError::InMemoryCheckpointIo { msg, source: _ } => GrpcError::Internal {
                message: format!("in-memory checkpoint io error: {}", msg),
            },
        }
    }
}

// Convert our grpc-local error enum into a tonic::Status so service handlers
// can return that directly to clients.
impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::MissingRequiredEnvVar { var } => {
                Status::failed_precondition(format!("missing required env var: {}", var))
            }
            GrpcError::InvalidDimension { .. } => Status::invalid_argument(e.to_string()),
            GrpcError::InvalidDataPath { source } => {
                Status::invalid_argument(format!("invalid data path: {}", source))
            }
            GrpcError::Io { source } => Status::internal(format!("io error: {}", source)),
            GrpcError::NotFound { message } => Status::not_found(message),
            GrpcError::InvalidArgument { message } => Status::invalid_argument(message),
            GrpcError::AlreadyExists { message } => Status::already_exists(message),
            GrpcError::FailedPrecondition { message } => Status::failed_precondition(message),
            GrpcError::Internal { message } => Status::internal(message),
        }
    }
}

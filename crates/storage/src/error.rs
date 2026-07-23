use defs::PointId;
use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum StorageError {
    #[snafu(display("Failed to open RocksDB at path '{path}': {source}"))]
    RocksDbOpen {
        path: String,
        source: rocksdb::Error,
    },

    #[snafu(display("Failed to intialize rocksdb"))]
    RocksDbInitialization {},

    #[snafu(display("Storage checkpoint error: {}", msg))]
    RocksDbCheckpointMsg { msg: String },

    #[snafu(display("{} : {}", msg, source))]
    RocksDbCheckpointIo { msg: String, source: std::io::Error },

    #[snafu(display("Failed to open rocksdb checkpoint: {source}"))]
    RocksDbCheckpoint { source: rocksdb::Error },

    #[snafu(display("Failed to flush database: {source}"))]
    RocksDbFlush { source: rocksdb::Error },

    #[snafu(display("Failed to read point {id} from storage: {source}"))]
    RocksDbRead { id: PointId, source: rocksdb::Error },

    #[snafu(display("Failed to write point {id} to storage: {source}"))]
    RocksDbWrite { id: PointId, source: rocksdb::Error },

    #[snafu(display("Failed to delete point {id} from storage: {source}"))]
    RocksDbDelete { id: PointId, source: rocksdb::Error },

    #[snafu(display("Failed to iterate over storage: {source}"))]
    RocksDbIteration { source: rocksdb::Error },

    #[snafu(display("Failed to lock in-memory storage"))]
    InMemoryLock {},

    #[snafu(display("In-memory checkpoint error: {}", msg))]
    InMemoryCheckpoint { msg: String },

    #[snafu(display("{} : {}", msg, source))]
    InMemoryCheckpointIo { msg: String, source: std::io::Error },

    #[snafu(display("Failed to serialize point {id}: {source}"))]
    Serialization { id: PointId, source: bincode::Error },

    #[snafu(display("Failed to deserialize point {id}: {source}"))]
    Deserialization { id: PointId, source: bincode::Error },
}

pub type Result<T, E = StorageError> = std::result::Result<T, E>;

use crate::StorageType;
use crate::in_memory::INMEMORY_CHECKPOINT_FILENAME_MARKER;
use crate::rocks_db::ROCKSDB_CHECKPOINT_FILENAME_MARKER;
use defs::DbError;
use std::path::{Path, PathBuf};

impl StorageType {
    #[inline]
    pub fn checkpoint_filename_marker(&self) -> &str {
        match self {
            StorageType::InMemory => INMEMORY_CHECKPOINT_FILENAME_MARKER,
            StorageType::RocksDb => ROCKSDB_CHECKPOINT_FILENAME_MARKER,
        }
    }
}

pub struct StorageCheckpoint {
    pub path: PathBuf,
    pub storage_type: StorageType,
}

impl StorageCheckpoint {
    pub fn open(path: &Path) -> Result<StorageCheckpoint, DbError> {
        let filename = path
            .file_name()
            .ok_or_else(|| DbError::StorageCheckpointError("Invalid filename".to_string()))?
            .to_str()
            .ok_or_else(|| {
                DbError::StorageCheckpointError("Invalid UTF-8 in filename".to_string())
            })?
            .to_owned();
        let marker = filename
            .split_once("-")
            .ok_or_else(|| DbError::StorageCheckpointError("Invalid filename".to_string()))?
            .0;

        let storage_type = match marker {
            INMEMORY_CHECKPOINT_FILENAME_MARKER => StorageType::InMemory,
            ROCKSDB_CHECKPOINT_FILENAME_MARKER => StorageType::RocksDb,
            _ => {
                return Err(DbError::StorageCheckpointError(
                    "Invalid storage type".to_string(),
                ));
            }
        };

        Ok(StorageCheckpoint {
            path: path.to_path_buf(),
            storage_type,
        })
    }
}

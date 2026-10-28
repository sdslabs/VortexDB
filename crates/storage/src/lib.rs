use crate::rocks_db::RocksDbStorage;
use defs::{DenseVector, Payload, Point, PointId, SparseVector};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
pub mod checkpoint;

pub type PointPage = (Vec<Point>, PointId);

pub mod error;
pub mod in_memory;
pub mod rocks_db;

pub use error::{Result, StorageError};

pub trait StorageEngine: Send + Sync {
    fn insert_point(
        &self,
        id: PointId,
        vector: Option<DenseVector>,
        sparse_vector: Option<SparseVector>,
        payload: Option<Payload>,
    ) -> Result<()>;
    fn get_point(&self, id: PointId) -> Result<Option<Point>>;
    fn get_vector(&self, id: PointId) -> Result<Option<DenseVector>>;
    fn delete_point(&self, id: PointId) -> Result<()>;
    fn contains_point(&self, id: PointId) -> Result<bool>;
    fn list_points(&self, offset: PointId, limit: usize) -> Result<Option<PointPage>>;

    fn checkpoint_at(&self, path: &Path) -> Result<checkpoint::StorageCheckpoint>;
    fn restore_checkpoint(&mut self, checkpoint: &checkpoint::StorageCheckpoint) -> Result<()>;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum StorageType {
    InMemory,
    RocksDb,
}

pub fn create_storage_engine(
    storage_type: StorageType,
    path: impl Into<PathBuf>,
) -> Result<Arc<dyn StorageEngine>> {
    match storage_type {
        StorageType::InMemory => Ok(Arc::new(in_memory::MemoryStorage::new())),
        StorageType::RocksDb => match RocksDbStorage::new(path) {
            Ok(rocks_db) => Ok(Arc::new(rocks_db)),
            Err(e) => Err(e),
        },
    }
}

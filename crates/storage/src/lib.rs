use defs::{DbError, DenseVector, Payload, PointId};
use std::path::PathBuf;
use std::sync::Arc;

use crate::rocks_db::RocksDbStorage;

pub type VectorPage = (Vec<(PointId, DenseVector)>, PointId);

pub trait StorageEngine: Send + Sync {
    fn insert_point(
        &self,
        id: PointId,
        vector: Option<DenseVector>,
        payload: Option<Payload>,
    ) -> Result<(), DbError>;
    fn get_vector(&self, id: PointId) -> Result<Option<DenseVector>, DbError>;
    fn get_payload(&self, id: PointId) -> Result<Option<Payload>, DbError>;
    fn delete_point(&self, id: PointId) -> Result<(), DbError>;
    fn contains_point(&self, id: PointId) -> Result<bool, DbError>;
    fn list_vectors(&self, offset: PointId, limit: usize) -> Result<Option<VectorPage>, DbError>;
}

pub mod in_memory;
pub mod rocks_db;

#[derive(Debug, Clone, Copy)]
pub enum StorageType {
    InMemory,
    RocksDb,
}

pub fn create_storage_engine(
    storage_type: StorageType,
    path: impl Into<PathBuf>,
) -> Result<Arc<dyn StorageEngine>, DbError> {
    match storage_type {
        StorageType::InMemory => Ok(Arc::new(in_memory::MemoryStorage::new())),
        StorageType::RocksDb => match RocksDbStorage::new(path) {
            Ok(rocks_db) => Ok(Arc::new(rocks_db)),
            Err(e) => Err(e),
        },
    }
}

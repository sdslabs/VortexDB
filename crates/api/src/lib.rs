use defs::{DbError, Dimension, IndexedVector, SearchQueryInput, Similarity, SnapshottableDb};
use defs::{DenseVector, Payload, Point, PointId, PointInput};
use index::hnsw::{HnswConfig, HnswIndex};
use index::kd_tree::{KDTree, KDTreeConfig};
use std::path::{Path, PathBuf};
use tempfile::tempdir;
// use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use index::flat::index::FlatIndex;
use index::{IndexType, VectorIndex};
use snapshot::Snapshot;
use storage::{StorageEngine, StorageType, VectorPage, create_storage_engine};

use uuid::Uuid;

pub mod error;
pub use error::{ApiError, Result};

// static NEXT_ID: AtomicU64 = AtomicU64::new(1);

// fn generate_point_id() -> u64 {
//     NEXT_ID.fetch_add(1, Ordering::Relaxed)
// }

fn generate_point_id() -> PointId {
    Uuid::new_v4()
}

pub struct VectorDb {
    storage: Arc<dyn StorageEngine>,
    index: Arc<RwLock<dyn VectorIndex>>, // Using a RwLock instead of Mutex to improve concurrency
    dimension: usize,
}

impl VectorDb {
    fn _new(
        storage: Arc<dyn StorageEngine>,
        index: Arc<RwLock<dyn VectorIndex>>,
        dimension: usize,
    ) -> Self {
        Self {
            storage,
            index,
            dimension,
        }
    }

    //TODO: Make this an atomic operation
    pub fn insert(&self, vector: DenseVector, payload: Payload) -> Result<PointId> {
        if vector.len() != self.dimension {
            return Err(ApiError::DimensionMismatch {
                expected: self.dimension,
                got: vector.len(),
            });
        }
        // Generate a new point id
        let point_id = generate_point_id();
        self.storage
            .insert_point(point_id, Some(vector.clone()), Some(payload))?;

        // Get write lock on the index
        let mut index = self.index.write().map_err(|_| ApiError::LockError)?;
        index.insert(IndexedVector {
            vector,
            id: point_id,
        })?;

        Ok(point_id)
    }

    pub fn insert_batch(&self, points: Vec<PointInput>) -> Result<Vec<PointId>> {
        let mut ids = Vec::with_capacity(points.len());

        for point in points {
            let id = point.id.unwrap_or_else(Uuid::new_v4);
            let vector = point.vector;
            let payload = point.payload;

            if let Some(ref v) = vector
                && v.len() != self.dimension
            {
                return Err(ApiError::DimensionMismatch {
                    expected: self.dimension,
                    got: v.len(),
                });
            }

            self.storage.insert_point(id, vector.clone(), payload)?;

            if let Some(v) = vector {
                let indexed = IndexedVector { id, vector: v };
                let mut index = self.index.write().map_err(|_| ApiError::LockError)?;
                index.insert(indexed)?;
            }

            ids.push(id);
        }

        Ok(ids)
    }

    //TODO: Make this an atomic operation
    pub fn delete(&self, id: PointId) -> Result<bool> {
        // Remove from storage
        self.storage.delete_point(id)?;
        // Remove from index
        let mut index = self.index.write().map_err(|_| ApiError::LockError)?;
        let point_found = index.delete(id)?;
        Ok(point_found)
    }

    pub fn get(&self, id: PointId) -> Result<Option<Point>> {
        // Search for the Point with given id in storage
        let payload = self.storage.get_payload(id)?;
        let vector = self.storage.get_vector(id)?;
        if payload.is_some() || vector.is_some() {
            Ok(Some(Point {
                id,
                payload,
                vector,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn search(&self, query: SearchQueryInput) -> Result<Vec<PointId>> {
        // Validate search limit
        if query.limit == 0 {
            return Err(ApiError::InvalidSearchLimit { limit: query.limit });
        }

        // Validate query dimension
        if query.vector.len() != self.dimension {
            return Err(ApiError::DimensionMismatch {
                expected: self.dimension,
                got: query.vector.len(),
            });
        }

        // Use vector index to find similar vectors
        let index = self.index.read().map_err(|_| ApiError::LockError)?;

        //TODO: Add feat of returning similarity scores in the search
        let vectors =
            index.search_with_ef(query.vector, query.similarity, query.limit, query.ef)?;

        Ok(vectors)
    }

    pub fn search_batch(&self, queries: Vec<SearchQueryInput>) -> Result<Vec<Vec<PointId>>> {
        let mut results = Vec::with_capacity(queries.len());
        let index = self.index.read().unwrap();

        for query in queries {
            let found =
                index.search_with_ef(query.vector, query.similarity, query.limit, query.ef)?;
            results.push(found);
        }

        Ok(results)
    }

    pub fn list(&self, offset: PointId, limit: usize) -> Result<Option<VectorPage>> {
        let page = self.storage.list_vectors(offset, limit)?;
        Ok(page)
    }

    // populates the current index with vectors from the storage
    pub fn build_index(&self) -> Result<usize> {
        // start from the minimal UUID and fetch in bounded batches and insert
        let mut offset = Uuid::nil();
        let page_size: usize = 1000;
        let mut inserted: usize = 0;

        let mut index = self.index.write().map_err(|_| ApiError::LockError)?;

        while let Some((batch, next_offset)) = self.storage.list_vectors(offset, page_size)? {
            if batch.is_empty() || next_offset == offset {
                break;
            }

            for (id, vector) in batch {
                index.insert(IndexedVector { id, vector })?;
                inserted += 1;
            }

            offset = next_offset;
        }

        Ok(inserted)
    }
}

impl SnapshottableDb for VectorDb {
    fn create_snapshot(&self, dir_path: &Path) -> Result<PathBuf, DbError> {
        if !dir_path.is_dir() {
            return Err(DbError::SnapshotError(format!(
                "Invalid path: {}",
                dir_path.display()
            )));
        }

        let index_snapshot = self
            .index
            .read()
            .map_err(|_| DbError::LockError)?
            .snapshot()?;

        let tempdir = tempdir().unwrap();
        let storage_checkpoint = self.storage.checkpoint_at(tempdir.path()).map_err(|e| {
            DbError::StorageCheckpointError(format!("Could not create storage checkpoint: {e}"))
        })?;

        let snapshot = Snapshot::new(index_snapshot, storage_checkpoint, self.dimension)?;
        let snapshot_path = snapshot.save(dir_path)?;

        Ok(snapshot_path)
    }
}

#[derive(Debug)]
pub struct DbConfig {
    pub storage_type: StorageType,
    pub index_type: IndexType,
    pub data_path: PathBuf,
    pub dimension: Dimension,
    pub similarity: Similarity,
    pub hnsw_config: HnswConfig,
    pub kd_tree_config: KDTreeConfig,
}

#[derive(Debug)]
pub struct DbRestoreConfig {
    pub data_path: PathBuf,
    pub snapshot_path: PathBuf,
}

impl DbRestoreConfig {
    pub fn new(data_path: PathBuf, snapshot_path: PathBuf) -> Self {
        Self {
            data_path,
            snapshot_path,
        }
    }
}

pub fn restore_from_snapshot(config: &DbRestoreConfig) -> Result<VectorDb, DbError> {
    // restore the index from the snapshot
    let (storage_engine, index, dimensions) =
        Snapshot::load(&config.snapshot_path, &config.data_path)?;
    Ok(VectorDb::_new(storage_engine, index, dimensions))
}

pub fn init_api(config: DbConfig) -> Result<VectorDb> {
    // Initialize the storage engine
    let storage = create_storage_engine(config.storage_type, config.data_path)?;

    // Initialize the vector index
    let index: Arc<RwLock<dyn VectorIndex>> = match config.index_type {
        IndexType::Flat => Arc::new(RwLock::new(FlatIndex::new())),
        IndexType::KDTree => Arc::new(RwLock::new(KDTree::build_empty_with_config(
            config.dimension,
            config.kd_tree_config,
        ))),
        IndexType::HNSW => Arc::new(RwLock::new(HnswIndex::with_config(
            config.similarity,
            config.dimension,
            config.hnsw_config,
        ))),
    };

    // Init the db
    let db = VectorDb::_new(storage, index, config.dimension);

    // populate the current index with vectors from the storage
    db.build_index()?;

    Ok(db)
}

#[cfg(test)]
mod tests;

use defs::{DbError, Dimension, IndexedVector, Similarity, SnapshottableDb};
use defs::{Point, PointId, PointInput, QueryVector, SearchHit, SearchQuery};
use index::hnsw::{HnswConfig, HnswIndex};
use index::inverted::InvertedIndex;
use index::kd_tree::{KDTree, KDTreeConfig};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
// use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use index::flat::index::FlatIndex;
use index::{IndexType, VectorIndex};
use snafu::ResultExt;
use snapshot::Snapshot;
use storage::{PointPage, StorageEngine, StorageType, create_storage_engine};

use uuid::Uuid;

pub mod error;
pub use error::{ApiError, InvalidSparseVectorSnafu, Result};

// static NEXT_ID: AtomicU64 = AtomicU64::new(1);

// fn generate_point_id() -> u64 {
//     NEXT_ID.fetch_add(1, Ordering::Relaxed)
// }

fn generate_point_id() -> PointId {
    Uuid::new_v4()
}

const MAX_SEARCH_LIMIT: usize = 10_000;

pub struct VectorDb {
    storage: Arc<dyn StorageEngine>,
    index: Arc<RwLock<dyn VectorIndex>>, // Using a RwLock instead of Mutex to improve concurrency
    sparse_index: RwLock<InvertedIndex>,
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
            sparse_index: RwLock::new(InvertedIndex::new()),
            dimension,
        }
    }

    fn validate_point(&self, point: &PointInput, id: PointId) -> Result<()> {
        if point.dense_vector.is_none() && point.sparse_vector.is_none() {
            return Err(ApiError::MissingVector { id });
        }

        if let Some(ref v) = point.dense_vector
            && v.len() != self.dimension
        {
            return Err(ApiError::DimensionMismatch {
                expected: self.dimension,
                got: v.len(),
            });
        }

        if let Some(ref sv) = point.sparse_vector {
            sv.validate().context(InvalidSparseVectorSnafu)?;
        }

        Ok(())
    }

    pub fn insert(&self, point: PointInput) -> Result<PointId> {
        let mut ids = self.insert_batch(vec![point])?;
        Ok(ids.pop().expect("insert_batch returns one id per point"))
    }

    //TODO: Make this an atomic operation
    pub fn insert_batch(&self, points: Vec<PointInput>) -> Result<Vec<PointId>> {
        // Validate the whole batch before writing anything
        let mut ids = Vec::with_capacity(points.len());
        let mut seen = HashSet::with_capacity(points.len());

        for point in &points {
            let id = point.id.unwrap_or_else(generate_point_id);
            self.validate_point(point, id)?;

            if !seen.insert(id) || (point.id.is_some() && self.storage.contains_point(id)?) {
                return Err(ApiError::PointAlreadyExists { id });
            }
            ids.push(id);
        }

        for (&id, point) in ids.iter().zip(points) {
            let PointInput {
                dense_vector,
                sparse_vector,
                payload,
                ..
            } = point;

            self.storage
                .insert_point(id, dense_vector.clone(), sparse_vector.clone(), payload)?;

            if let Some(vector) = dense_vector {
                let mut index = self.index.write().map_err(|_| ApiError::LockError)?;
                index.insert(IndexedVector { id, vector })?;
            }
            if let Some(sv) = sparse_vector {
                let mut sparse_index =
                    self.sparse_index.write().map_err(|_| ApiError::LockError)?;
                sparse_index.insert(id, &sv);
            }
        }

        Ok(ids)
    }

    //TODO: Make this an atomic operation
    pub fn delete(&self, id: PointId) -> Result<bool> {
        let Some(point) = self.storage.get_point(id)? else {
            return Ok(false);
        };

        self.storage.delete_point(id)?;

        if point.dense_vector.is_some() {
            let mut index = self.index.write().map_err(|_| ApiError::LockError)?;
            index.delete(id)?;
        }
        if let Some(sv) = point.sparse_vector {
            let mut sparse_index = self.sparse_index.write().map_err(|_| ApiError::LockError)?;
            sparse_index.delete(id, &sv);
        }

        Ok(true)
    }

    pub fn get(&self, id: PointId) -> Result<Option<Point>> {
        Ok(self.storage.get_point(id)?)
    }

    pub fn search(&self, query: SearchQuery) -> Result<Vec<SearchHit>> {
        if query.limit == 0 || query.limit > MAX_SEARCH_LIMIT {
            return Err(ApiError::InvalidSearchLimit { limit: query.limit });
        }

        let results = match query.query {
            QueryVector::Dense(dense) => {
                if dense.vector.len() != self.dimension {
                    return Err(ApiError::DimensionMismatch {
                        expected: self.dimension,
                        got: dense.vector.len(),
                    });
                }
                let index = self.index.read().map_err(|_| ApiError::LockError)?;
                index.search_with_ef(dense.vector, dense.similarity, query.limit, dense.ef)?
            }
            QueryVector::Sparse(sparse_vector) => {
                sparse_vector.validate().context(InvalidSparseVectorSnafu)?;
                let sparse_index = self.sparse_index.read().map_err(|_| ApiError::LockError)?;
                sparse_index.search(&sparse_vector, query.limit)
            }
        };

        Ok(results
            .into_iter()
            .map(|(id, score)| SearchHit { id, score })
            .collect())
    }

    pub fn search_batch(&self, queries: Vec<SearchQuery>) -> Result<Vec<Vec<SearchHit>>> {
        queries
            .into_iter()
            .map(|query| self.search(query))
            .collect()
    }

    pub fn list(&self, offset: PointId, limit: usize) -> Result<Option<PointPage>> {
        let page = self.storage.list_points(offset, limit)?;
        Ok(page)
    }

    // populates both indexes with the points in storage
    pub fn build_index(&self) -> Result<usize> {
        // start from the minimal UUID and fetch in bounded batches and insert
        let mut offset = Uuid::nil();
        let page_size: usize = 1000;
        let mut inserted: usize = 0;

        let mut index = self.index.write().map_err(|_| ApiError::LockError)?;
        let mut sparse_index = self.sparse_index.write().map_err(|_| ApiError::LockError)?;

        while let Some((batch, next_offset)) = self.storage.list_points(offset, page_size)? {
            if batch.is_empty() || next_offset == offset {
                break;
            }

            for Point {
                id,
                dense_vector,
                sparse_vector,
                ..
            } in batch
            {
                if let Some(vector) = dense_vector {
                    index.insert(IndexedVector { id, vector })?;
                }
                if let Some(sv) = sparse_vector {
                    sparse_index.insert(id, &sv);
                }
                inserted += 1;
            }

            offset = next_offset;
        }

        Ok(inserted)
    }

    // populates only the sparse index, used after a snapshot restore where the dense index comes from the snapshot
    fn build_sparse_index(&self) -> Result<()> {
        let mut offset = Uuid::nil();
        let page_size: usize = 1000;

        let mut sparse_index = self.sparse_index.write().map_err(|_| ApiError::LockError)?;

        while let Some((batch, next_offset)) = self.storage.list_points(offset, page_size)? {
            if batch.is_empty() || next_offset == offset {
                break;
            }

            for point in batch {
                if let Some(sv) = point.sparse_vector {
                    sparse_index.insert(point.id, &sv);
                }
            }

            offset = next_offset;
        }

        Ok(())
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
    let db = VectorDb::_new(storage_engine, index, dimensions);
    db.build_sparse_index()
        .map_err(|e| DbError::IndexError(e.to_string()))?;
    Ok(db)
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

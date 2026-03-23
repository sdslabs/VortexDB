use defs::{DbError, Dimension, IndexedVector, SearchQueryInput, Similarity, SnapshottableDb};
use defs::{DenseVector, Payload, Point, PointId, PointInput};
use index::hnsw::HnswIndex;
use index::kd_tree::index::KDTree;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
// use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use index::flat::index::FlatIndex;
use index::{IndexType, VectorIndex};
use snapshot::Snapshot;
use storage::rocks_db::RocksDbStorage;
use storage::{StorageEngine, StorageType, VectorPage};

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
        let vectors = index.search(query.vector, query.similarity, query.limit)?;

        Ok(vectors)
    }

    pub fn search_batch(&self, queries: Vec<SearchQueryInput>) -> Result<Vec<Vec<PointId>>> {
        let mut results = Vec::with_capacity(queries.len());
        let index = self.index.read().unwrap();

        for query in queries {
            let found = index.search(query.vector, query.similarity, query.limit)?;
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
    let storage = match config.storage_type {
        StorageType::RocksDb => Arc::new(RocksDbStorage::new(config.data_path)?),
        _ => Arc::new(RocksDbStorage::new(config.data_path)?),
    };

    // Initialize the vector index
    let index: Arc<RwLock<dyn VectorIndex>> = match config.index_type {
        IndexType::Flat => Arc::new(RwLock::new(FlatIndex::new())),
        IndexType::KDTree => Arc::new(RwLock::new(KDTree::build_empty(config.dimension))),
        IndexType::HNSW => Arc::new(RwLock::new(HnswIndex::new(
            config.similarity,
            config.dimension,
        ))),
    };

    // Init the db
    let db = VectorDb::_new(storage, index, config.dimension);

    // populate the current index with vectors from the storage
    db.build_index()?;

    Ok(db)
}

#[cfg(test)]
mod tests {

    // TODO: Add more exhaustive tests

    use std::sync::Mutex;

    use super::*;
    use defs::ContentType;
    use snapshot::{engine::SnapshotEngine, registry::local::LocalRegistry};
    use tempfile::{TempDir, tempdir};

    // Helper function to create a test database
    fn create_test_db() -> (VectorDb, TempDir) {
        let temp_dir = tempdir().unwrap();
        let config = DbConfig {
            storage_type: StorageType::RocksDb,
            index_type: IndexType::Flat,
            data_path: temp_dir.path().to_path_buf(),
            dimension: 3,
            similarity: Similarity::Cosine,
        };
        (init_api(config).unwrap(), temp_dir)
    }

    #[test]
    fn test_insert_and_get() {
        let (db, _temp_dir) = create_test_db();
        let vector = vec![1.0, 2.0, 3.0];
        let payload = Payload {
            content_type: ContentType::Text,
            content: "Test content".to_string(),
        };

        // Test insert
        let id = db.insert(vector.clone(), payload.clone()).unwrap();
        assert!(id != Uuid::nil());

        // Test get
        let point = db.get(id).unwrap().unwrap();
        assert_eq!(point.id, id);
        assert_eq!(point.vector.as_ref().unwrap(), &vector);
        assert_eq!(point.payload.as_ref().unwrap(), &payload);
        assert_eq!(
            point.payload.as_ref().unwrap().content_type,
            ContentType::Text
        );
        assert_eq!(point.payload.as_ref().unwrap().content, "Test content");
    }

    #[test]
    fn test_dimension_mismatch() {
        let (db, _temp_dir) = create_test_db();
        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![1.0, 2.0];
        let payload = defs::Payload {
            content_type: ContentType::Text,
            content: "tester".to_string(),
        };

        let res1 = db.insert(v1, payload.clone());
        assert!(res1.is_ok());

        // Insert vector of dimension 2 != 3
        let res2 = db.insert(v2, payload);
        assert!(res2.is_err());
        match res2.unwrap_err() {
            ApiError::DimensionMismatch { expected, got } => {
                assert_eq!(expected, 3);
                assert_eq!(got, 2);
            }
            other => panic!("Expected DimensionMismatch, got: {:?}", other),
        }
    }

    #[test]
    fn test_delete() {
        let (db, _temp_dir) = create_test_db();
        let vector = vec![1.0, 2.0, 3.0];
        let payload = Payload {
            content_type: ContentType::Text,
            content: "Test content".to_string(),
        };

        // Insert a point
        let id = db.insert(vector, payload).unwrap();

        // try deleting a point that does not exist
        let found = db.delete(Uuid::new_v4());
        assert!(found.is_ok());
        assert!(!found.unwrap());

        // delete the point
        assert!(db.get(id).unwrap().is_some());
        db.delete(id).unwrap();
        assert!(db.get(id).unwrap().is_none());
    }

    #[test]
    fn test_search() {
        let (db, _temp_dir) = create_test_db();

        // Insert some points
        let vectors = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];

        let mut ids = Vec::new();
        for vector in vectors {
            let payload = Payload {
                content_type: ContentType::Text,
                content: format!("Test content {vector:?}"),
            };
            let id = db.insert(vector, payload).unwrap();
            ids.push(id);
        }

        // Search for the closest vector to [1.0, 0.1, 0.1]
        let query = vec![1.0, 0.1, 0.1];
        let results = db
            .search(SearchQueryInput {
                vector: query,
                similarity: Similarity::Cosine,
                limit: 1,
            })
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], ids[0]); // The first vector should be closest
    }

    #[test]
    fn test_search_limit() {
        let (db, _temp_dir) = create_test_db();

        // Insert 5 points
        let mut ids = Vec::new();
        for i in 0..5 {
            let vector = vec![i as f32, 0.0, 0.0];
            let id = db
                .insert(
                    vector,
                    Payload {
                        content_type: ContentType::Text,
                        content: format!("Test content {i}"),
                    },
                )
                .unwrap();
            ids.push(id);
        }

        // Search with limit 3
        let query = vec![0.0, 0.0, 0.0];
        let results = db
            .search(SearchQueryInput {
                vector: query,
                similarity: Similarity::Euclidean,
                limit: 3,
            })
            .unwrap();

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_search_zero_limit() {
        let (db, _temp_dir) = create_test_db();

        let query = vec![1.0, 2.0, 3.0];
        let result = db.search(SearchQueryInput {
            vector: query,
            similarity: Similarity::Cosine,
            limit: 0,
        });

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvalidSearchLimit { limit } => {
                assert_eq!(limit, 0);
            }
            other => panic!("Expected InvalidSearchLimit, got: {:?}", other),
        }
    }

    #[test]
    fn test_empty_database() {
        let (db, _temp_dir) = create_test_db();

        // Get non-existent point
        assert!(db.get(Uuid::new_v4()).unwrap().is_none());

        let query = vec![1.0, 2.0, 3.0];
        let results = db
            .search(SearchQueryInput {
                vector: query,
                similarity: Similarity::Cosine,
                limit: 10,
            })
            .unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_list_vectors() {
        let (db, _temp_dir) = create_test_db();
        // insert some points
        let mut ids = Vec::new();
        for i in 0..10 {
            let i = i as f32;
            let vector = vec![i, i + 1.0, i + 2.0];
            let id = db
                .insert(
                    vector,
                    Payload {
                        content_type: ContentType::Text,
                        content: format!("Test content {i}"),
                    },
                )
                .unwrap();
            ids.push(id);
        }

        // list vectors with limit 5
        // list the values as well as their length
        let (vectors, next_offset) = db.list(Uuid::nil(), 5).unwrap().unwrap();
        assert_eq!(vectors.len(), 5);

        // list next set of vectors
        // list the values as well as their length
        let (next_vectors, _) = db.list(next_offset, 5).unwrap().unwrap();
        assert_eq!(next_vectors.len(), 5);
    }

    #[test]
    fn test_build_index() {
        let (db, _temp_dir) = create_test_db();

        // insert some points
        for i in 0..10 {
            let i = i as f32;
            let vector = vec![i, i + 1.0, i + 2.0];
            db.insert(
                vector,
                Payload {
                    content_type: ContentType::Text,
                    content: format!("Test content {i}"),
                },
            )
            .unwrap();
        }

        // rebuild the index
        let inserted = db.build_index().unwrap();
        assert_eq!(inserted, 10);
    }

    #[test]
    fn test_create_and_load_snapshot() {
        let (old_db, temp_dir) = create_test_db();

        let v1 = vec![0.0, 1.0, 2.0];
        let v2 = vec![3.0, 4.0, 5.0];
        let v3 = vec![6.0, 7.0, 8.0];

        let id1 = old_db
            .insert(
                v1.clone(),
                Payload {
                    content_type: ContentType::Text,
                    content: "test".to_string(),
                },
            )
            .unwrap();

        let id2 = old_db
            .insert(
                v2.clone(),
                Payload {
                    content_type: ContentType::Text,
                    content: "test".to_string(),
                },
            )
            .unwrap();

        let temp_snapshot_dir = tempdir().unwrap();
        let snapshot_path = old_db.create_snapshot(temp_snapshot_dir.path()).unwrap();

        // insert v3 after snapshot
        let id3 = old_db
            .insert(
                v3.clone(),
                Payload {
                    content_type: ContentType::Text,
                    content: "test".to_string(),
                },
            )
            .unwrap();

        let reload_config = DbRestoreConfig {
            data_path: temp_dir.path().to_path_buf(),
            snapshot_path,
        };

        std::mem::drop(old_db);
        let loaded_db = restore_from_snapshot(&reload_config).unwrap();

        assert!(loaded_db.get(id1).unwrap_or(None).is_some());
        assert!(loaded_db.get(id2).unwrap_or(None).is_some());
        assert!(loaded_db.get(id3).unwrap_or(None).is_none()); // v3 was inserted after snapshot was taken

        // vector restore check
        assert!(loaded_db.get(id1).unwrap().unwrap().vector.unwrap() == v1);
        assert!(loaded_db.get(id2).unwrap().unwrap().vector.unwrap() == v2);
    }

    #[test]
    fn test_snapshot_engine() {
        let (_db, _temp_dir) = create_test_db();
        let db = Arc::new(Mutex::new(_db));

        let registry_tempdir = tempdir().unwrap();

        let registry = Arc::new(Mutex::new(
            LocalRegistry::new(registry_tempdir.path()).unwrap(),
        ));

        let last_k = 4;
        let mut se = SnapshotEngine::new(last_k, db.clone(), registry.clone());

        let v1 = vec![0.0, 1.0, 2.0];
        let v2 = vec![3.0, 4.0, 5.0];
        let v3 = vec![6.0, 7.0, 8.0];

        let test_vectors = vec![v1.clone(), v2.clone(), v3.clone()];
        let mut inserted_ids = Vec::new();

        for (i, vector) in test_vectors.clone().into_iter().enumerate() {
            se.snapshot().unwrap();
            let id = db
                .lock()
                .unwrap()
                .insert(
                    vector.clone(),
                    Payload {
                        content_type: ContentType::Text,
                        content: format!("{}", i),
                    },
                )
                .unwrap();
            inserted_ids.push(id);
        }
        se.snapshot().unwrap();
        let snapshots = se.list_alive_snapshots().unwrap();

        // asserting these cases:
        // snapshot 0 : no vectors
        // snapshot 1 : v1
        // snapshot 2 : v1, v2
        // snapshot 3 : v1, v2, v3

        std::mem::drop(db);
        std::mem::drop(se);

        for (i, snapshot) in snapshots.iter().enumerate() {
            let temp_dir = tempdir().unwrap();
            let db = restore_from_snapshot(&DbRestoreConfig {
                data_path: temp_dir.path().to_path_buf(),
                snapshot_path: snapshot.path.clone(),
            })
            .unwrap();
            for j in 0..i {
                // test if point is present
                assert!(db.get(inserted_ids[j]).unwrap_or(None).is_some());
                // test vector restore
                assert!(
                    db.get(inserted_ids[j]).unwrap().unwrap().vector.unwrap() == test_vectors[j]
                );
                // test payload restore
                assert!(
                    db.get(inserted_ids[j])
                        .unwrap()
                        .unwrap()
                        .payload
                        .unwrap()
                        .content
                        == format!("{}", j)
                );
            }
            for absent_id in inserted_ids.iter().skip(i) {
                assert!(db.get(*absent_id).unwrap_or(None).is_none());
            }
            std::mem::drop(db);
        }
    }
}

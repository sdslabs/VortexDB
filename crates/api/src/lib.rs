use defs::{DbError, IndexedVector, Similarity};

use defs::{DenseVector, Payload, Point, PointId};
use std::path::PathBuf;
// use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use index::flat::FlatIndex;
use index::{IndexType, VectorIndex};
use storage::rocks_db::RocksDbStorage;
use storage::{StorageEngine, StorageType, VectorPage};

use uuid::Uuid;

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
    pub fn insert(&self, vector: DenseVector, payload: Payload) -> Result<PointId, DbError> {
        if vector.len() != self.dimension {
            return Err(DbError::DimensionMismatch);
        }
        // Generate a new point id
        let point_id = generate_point_id();
        self.storage
            .insert_point(point_id, Some(vector.clone()), Some(payload))?;

        // Get write lock on the index
        let mut index = self.index.write().map_err(|_| DbError::LockError)?;
        index.insert(IndexedVector {
            vector,
            id: point_id,
        })?;

        Ok(point_id)
    }

    //TODO: Make this an atomic operation
    pub fn delete(&self, id: PointId) -> Result<bool, DbError> {
        // Remove from storage
        self.storage.delete_point(id)?;
        // Remove from index
        let mut index = self.index.write().map_err(|_| DbError::LockError)?;
        let point_found = index.delete(id)?;
        Ok(point_found)
    }

    pub fn get(&self, id: PointId) -> Result<Option<Point>, DbError> {
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

    pub fn search(
        &self,
        query: DenseVector,
        similarity: Similarity,
        limit: usize,
    ) -> Result<Vec<PointId>, DbError> {
        // Use vector index to find similar vectors
        let index = self.index.read().map_err(|_| DbError::LockError)?;

        //TODO: Add feat of returning similarity scores in the search
        let vectors = index.search(query, similarity, limit)?;

        Ok(vectors)
    }

    pub fn list(&self, offset: PointId, limit: usize) -> Result<Option<VectorPage>, DbError> {
        self.storage.list_vectors(offset, limit)
    }

    // populates the current index with vectors from the storage
    pub fn build_index(&self) -> Result<usize, DbError> {
        // start from the minimal UUID and fetch in bounded batches and insert
        let mut offset = Uuid::nil();
        let page_size: usize = 1000;
        let mut inserted: usize = 0;

        let mut index = self.index.write().map_err(|_| DbError::LockError)?;

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

#[derive(Debug)]
pub struct DbConfig {
    pub storage_type: StorageType,
    pub index_type: IndexType,
    pub data_path: PathBuf,
    pub dimension: usize,
}

pub fn init_api(config: DbConfig) -> Result<VectorDb, DbError> {
    // Initialize the storage engine
    let storage = match config.storage_type {
        StorageType::RocksDb => Arc::new(RocksDbStorage::new(config.data_path)?),
        _ => Arc::new(RocksDbStorage::new(config.data_path)?),
    };

    // Initialize the vector index
    let index: Arc<RwLock<dyn VectorIndex>> = match config.index_type {
        IndexType::Flat => Arc::new(RwLock::new(FlatIndex::new())),
        _ => Arc::new(RwLock::new(FlatIndex::new())),
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

    use super::*;
    use defs::ContentType;
    use tempfile::tempdir;

    // Helper function to create a test database
    fn create_test_db() -> VectorDb {
        let temp_dir = tempdir().unwrap();
        let config = DbConfig {
            storage_type: StorageType::RocksDb,
            index_type: IndexType::Flat,
            data_path: temp_dir.path().to_path_buf(),
            dimension: 3,
        };
        init_api(config).unwrap()
    }

    #[test]
    fn test_insert_and_get() {
        let db = create_test_db();
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
        let db = create_test_db();
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
        assert_eq!(res2.unwrap_err(), DbError::DimensionMismatch);
    }

    #[test]
    fn test_delete() {
        let db = create_test_db();
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
        let db = create_test_db();

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
        let results = db.search(query, Similarity::Cosine, 1).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], ids[0]); // The first vector should be closest
    }

    #[test]
    fn test_search_limit() {
        let db = create_test_db();

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
        let results = db.search(query, Similarity::Euclidean, 3).unwrap();

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_empty_database() {
        let db = create_test_db();

        // Get non-existent point
        assert!(db.get(Uuid::new_v4()).unwrap().is_none());

        let query = vec![1.0, 2.0, 3.0];
        let results = db.search(query, Similarity::Cosine, 10).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_list_vectors() {
        let db = create_test_db();
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
        let db = create_test_db();

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
}

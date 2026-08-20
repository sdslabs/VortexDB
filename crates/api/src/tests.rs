use super::*;

// TODO: Add more exhaustive tests

use std::sync::Mutex;

use defs::ContentType;
use snapshot::{engine::SnapshotEngine, registry::local::LocalRegistry};
use tempfile::{TempDir, tempdir};

// Helper function to create a test database
fn create_test_db() -> (VectorDb, TempDir) {
    create_test_db_with_storage(StorageType::RocksDb)
}

fn create_test_db_with_storage(storage_type: StorageType) -> (VectorDb, TempDir) {
    let temp_dir = tempdir().unwrap();
    let config = DbConfig {
        storage_type,
        index_type: IndexType::Flat,
        data_path: temp_dir.path().to_path_buf(),
        dimension: 3,
        similarity: Similarity::Cosine,
        hnsw_config: HnswConfig::default(),
        kd_tree_config: KDTreeConfig::default(),
    };
    (init_api(config).unwrap(), temp_dir)
}

fn test_payload(content: &str) -> Payload {
    Payload {
        content_type: ContentType::Text,
        content: content.to_string(),
    }
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
fn test_insert_and_get_with_in_memory_storage() {
    let (db, _temp_dir) = create_test_db_with_storage(StorageType::InMemory);
    let vector = vec![1.0, 2.0, 3.0];
    let payload = test_payload("Test content");

    let id = db.insert(vector.clone(), payload.clone()).unwrap();
    let point = db.get(id).unwrap().unwrap();

    assert_eq!(point.id, id);
    assert_eq!(point.vector, Some(vector));
    assert_eq!(point.payload, Some(payload));
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
            ef: None,
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
            ef: None,
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
        ef: None,
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
            ef: None,
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
fn test_create_and_load_snapshot_with_in_memory_storage() {
    let (old_db, temp_dir) = create_test_db_with_storage(StorageType::InMemory);

    let v1 = vec![0.0, 1.0, 2.0];
    let v2 = vec![3.0, 4.0, 5.0];
    let v3 = vec![6.0, 7.0, 8.0];

    let id1 = old_db.insert(v1.clone(), test_payload("one")).unwrap();
    let id2 = old_db.insert(v2.clone(), test_payload("two")).unwrap();

    let temp_snapshot_dir = tempdir().unwrap();
    let snapshot_path = old_db.create_snapshot(temp_snapshot_dir.path()).unwrap();

    let id3 = old_db.insert(v3, test_payload("three")).unwrap();

    let reload_config = DbRestoreConfig {
        data_path: temp_dir.path().to_path_buf(),
        snapshot_path,
    };

    let loaded_db = restore_from_snapshot(&reload_config).unwrap();

    assert_eq!(loaded_db.get(id1).unwrap().unwrap().vector, Some(v1));
    assert_eq!(loaded_db.get(id2).unwrap().unwrap().vector, Some(v2));
    assert!(loaded_db.get(id3).unwrap().is_none());
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
            assert!(db.get(inserted_ids[j]).unwrap().unwrap().vector.unwrap() == test_vectors[j]);
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

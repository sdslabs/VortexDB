use super::*;

use defs::ContentType;
use uuid::Uuid;

use tempfile::{TempDir, tempdir};

fn create_test_db() -> (RocksDbStorage, TempDir) {
    let temp_dir = tempdir().unwrap();

    let db = RocksDbStorage::new(temp_dir.path()).expect("Failed to create RocksDB");
    (db, temp_dir)
}

#[test]
fn test_new_rocksdb_storage() {
    let (db, temp_dir) = create_test_db();
    assert_eq!(db.get_current_path(), temp_dir.path());
}

#[test]
fn test_insert_and_get_vector() {
    let (db, _temp_dir) = create_test_db();
    let id = Uuid::new_v4();
    let vector = Some(vec![0.1, 0.2, 0.3]);
    let payload = Some(Payload {
        content_type: ContentType::Text,
        content: "Test".to_string(),
    });

    assert!(db.insert_point(id, vector.clone(), None, payload).is_ok());
    let result = db.get_vector(id).unwrap();
    assert_eq!(result, vector);
}

#[test]
fn test_insert_and_get_sparse_vector() {
    let (db, _temp_dir) = create_test_db();
    let id = Uuid::new_v4();
    let sparse_vector = Some(SparseVector {
        token_ids: vec![1, 5, 9],
        weights: vec![0.5, 1.0, 2.0],
    });

    db.insert_point(id, None, sparse_vector.clone(), None)
        .unwrap();
    assert_eq!(
        db.get_point(id)
            .unwrap()
            .and_then(|point| point.sparse_vector),
        sparse_vector
    );
    assert_eq!(db.get_vector(id).unwrap(), None);
}

#[test]
fn test_list_points_pages_dense_and_sparse_points() {
    let (db, _temp_dir) = create_test_db();
    let ids = [Uuid::from_u128(1), Uuid::from_u128(2), Uuid::from_u128(3)];
    let sparse_vector = SparseVector {
        token_ids: vec![2, 4],
        weights: vec![1.0, 0.5],
    };

    db.insert_point(ids[0], Some(vec![0.1, 0.2]), None, None)
        .unwrap();
    db.insert_point(ids[1], None, Some(sparse_vector.clone()), None)
        .unwrap();
    db.insert_point(ids[2], Some(vec![0.3, 0.4]), None, None)
        .unwrap();

    let (first_page, next_offset) = db.list_points(Uuid::nil(), 2).unwrap().unwrap();
    let first_ids: Vec<PointId> = first_page.iter().map(|point| point.id).collect();
    assert_eq!(first_ids, vec![ids[0], ids[1]]);
    assert_eq!(first_page[1].sparse_vector, Some(sparse_vector));

    let (second_page, _) = db.list_points(next_offset, 2).unwrap().unwrap();
    let second_ids: Vec<PointId> = second_page.iter().map(|point| point.id).collect();
    assert_eq!(second_ids, vec![ids[2]]);
}

#[test]
fn test_insert_and_get_payload() {
    let (db, _temp_dir) = create_test_db();
    let id = Uuid::new_v4();
    let payload = Some(Payload {
        content_type: ContentType::Text,
        content: "Test".to_string(),
    });
    let vector = None;

    // Move payload into insert_point and recreate expected for comparison
    assert!(db.insert_point(id, vector, None, payload).is_ok());
    let result = db.get_point(id).unwrap().and_then(|point| point.payload);
    let expected = Some(Payload {
        content_type: ContentType::Text,
        content: "Test".to_string(),
    });
    assert_eq!(result, expected);
}

#[test]
fn test_contains_point() {
    let (db, _temp_dir) = create_test_db();
    let id = Uuid::new_v4();
    let payload = Some(Payload {
        content_type: ContentType::Text,
        content: "Test".to_string(),
    });

    assert!(!db.contains_point(id).unwrap());

    let vector = Some(vec![0.4, 0.5, 0.6]);
    db.insert_point(id, vector, None, payload).unwrap();

    assert!(db.contains_point(id).unwrap());
}

#[test]
fn test_delete_point() {
    let (db, _temp_dir) = create_test_db();
    let id = Uuid::new_v4();
    let payload = Some(Payload {
        content_type: ContentType::Text,
        content: "Test".to_string(),
    });

    let vector = vec![0.7, 0.8, 0.9];

    db.insert_point(id, Some(vector), None, payload).unwrap();

    assert!(db.contains_point(id).unwrap());

    db.delete_point(id).unwrap();

    assert!(!db.contains_point(id).unwrap());
    assert_eq!(db.get_vector(id).unwrap(), None);
    assert_eq!(
        db.get_point(id).unwrap().and_then(|point| point.payload),
        None
    );
}

#[test]
fn test_get_nonexistent_vector() {
    let (db, _temp_dir) = create_test_db();
    let id = Uuid::new_v4();

    assert_eq!(db.get_vector(id).unwrap(), None);
}

#[test]
fn test_get_nonexistent_payload() {
    let (db, _temp_dir) = create_test_db();
    let id = Uuid::new_v4();

    assert_eq!(
        db.get_point(id).unwrap().and_then(|point| point.payload),
        None
    );
}
#[test]
fn test_error_context_preservation() {
    // Test that the error chain is preserved
    let result = RocksDbStorage::new("/proc/invalid-path");

    if let Err(err) = result {
        // The Display implementation should show both the context and source
        let err_string = format!("{}", err);
        println!("Full error message: {}", err_string);

        // Should contain our custom context
        assert!(err_string.contains("Failed to open RocksDB"));
        assert!(err_string.contains("/proc/invalid-path"));

        // The error should also be debuggable
        let debug_string = format!("{:?}", err);
        println!("Debug format: {}", debug_string);
    }
}

#[test]
fn test_create_and_load_checkpoint() {
    let (mut db, temp_dir) = create_test_db();

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    let vector = Some(vec![0.1, 0.2, 0.3]);
    let payload = Some(Payload {
        content_type: ContentType::Text,
        content: "Test".to_string(),
    });

    assert!(
        db.insert_point(id1, vector.clone(), None, payload.clone())
            .is_ok()
    );

    let checkpoint = db
        .checkpoint_at(temp_dir.path())
        .expect("Failed to create checkpoint");

    assert!(
        db.insert_point(id2, vector.clone(), None, payload.clone())
            .is_ok()
    );

    db.restore_checkpoint(&checkpoint).unwrap();

    assert!(db.contains_point(id1).unwrap());
    assert!(!db.contains_point(id2).unwrap());
}

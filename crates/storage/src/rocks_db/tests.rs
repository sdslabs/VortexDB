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

    assert!(db.insert_point(id, vector.clone(), payload).is_ok());
    let result = db.get_vector(id).unwrap();
    assert_eq!(result, vector);
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
    assert!(db.insert_point(id, vector, payload).is_ok());
    let result = db.get_payload(id).unwrap();
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
    db.insert_point(id, vector, payload).unwrap();

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

    db.insert_point(id, Some(vector), payload).unwrap();

    assert!(db.contains_point(id).unwrap());

    db.delete_point(id).unwrap();

    assert!(!db.contains_point(id).unwrap());
    assert_eq!(db.get_vector(id).unwrap(), None);
    assert_eq!(db.get_payload(id).unwrap(), None);
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

    assert_eq!(db.get_payload(id).unwrap(), None);
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
        db.insert_point(id1, vector.clone(), payload.clone())
            .is_ok()
    );

    let checkpoint = db
        .checkpoint_at(temp_dir.path())
        .expect("Failed to create checkpoint");

    assert!(
        db.insert_point(id2, vector.clone(), payload.clone())
            .is_ok()
    );

    db.restore_checkpoint(&checkpoint).unwrap();

    assert!(db.contains_point(id1).unwrap());
    assert!(!db.contains_point(id2).unwrap());
}

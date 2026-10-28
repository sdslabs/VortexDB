use super::*;

use defs::ContentType;
use std::io::{Read, Write};
use tempfile::{TempDir, tempdir};
use uuid::Uuid;

fn create_test_storage() -> MemoryStorage {
    MemoryStorage::new()
}

fn test_payload(content: &str) -> Payload {
    Payload {
        content_type: ContentType::Text,
        content: content.to_string(),
    }
}

#[test]
fn test_insert_and_get_vector() {
    let storage = create_test_storage();
    let id = Uuid::new_v4();
    let vector = Some(vec![0.1, 0.2, 0.3]);
    let payload = Some(test_payload("Test"));

    storage
        .insert_point(id, vector.clone(), None, payload)
        .unwrap();

    assert_eq!(storage.get_vector(id).unwrap(), vector);
}

#[test]
fn test_insert_and_get_payload() {
    let storage = create_test_storage();
    let id = Uuid::new_v4();
    let payload = Some(test_payload("Test"));

    storage
        .insert_point(id, None, None, payload.clone())
        .unwrap();

    assert_eq!(
        storage
            .get_point(id)
            .unwrap()
            .and_then(|point| point.payload),
        payload
    );
}

#[test]
fn test_contains_and_delete_point() {
    let storage = create_test_storage();
    let id = Uuid::new_v4();

    assert!(!storage.contains_point(id).unwrap());

    storage
        .insert_point(
            id,
            Some(vec![0.4, 0.5, 0.6]),
            None,
            Some(test_payload("Test")),
        )
        .unwrap();
    assert!(storage.contains_point(id).unwrap());

    storage.delete_point(id).unwrap();
    assert!(!storage.contains_point(id).unwrap());
    assert_eq!(storage.get_vector(id).unwrap(), None);
    assert_eq!(
        storage
            .get_point(id)
            .unwrap()
            .and_then(|point| point.payload),
        None
    );
}

#[test]
fn test_list_points_respects_offset_and_limit() {
    let storage = create_test_storage();
    let ids = [
        Uuid::from_u128(1),
        Uuid::from_u128(2),
        Uuid::from_u128(3),
        Uuid::from_u128(4),
    ];
    let sparse_vector = SparseVector {
        token_ids: vec![7, 9],
        weights: vec![0.5, 1.5],
    };

    storage
        .insert_point(ids[0], Some(vec![1.0, 1.1]), None, None)
        .unwrap();
    storage
        .insert_point(ids[1], None, Some(sparse_vector.clone()), None)
        .unwrap();
    storage
        .insert_point(ids[2], None, None, Some(test_payload("payload-only")))
        .unwrap();
    storage
        .insert_point(ids[3], Some(vec![4.0, 4.1]), None, None)
        .unwrap();

    let (first_page, next_offset) = storage.list_points(Uuid::nil(), 2).unwrap().unwrap();
    let first_ids: Vec<PointId> = first_page.iter().map(|point| point.id).collect();
    assert_eq!(first_ids, vec![ids[0], ids[1]]);
    assert_eq!(first_page[1].sparse_vector, Some(sparse_vector));
    assert_eq!(next_offset, ids[1]);

    let (second_page, next_offset) = storage.list_points(next_offset, 2).unwrap().unwrap();
    let second_ids: Vec<PointId> = second_page.iter().map(|point| point.id).collect();
    assert_eq!(second_ids, vec![ids[2], ids[3]]);
    assert_eq!(next_offset, ids[3]);

    let (empty_page, next_offset) = storage.list_points(next_offset, 2).unwrap().unwrap();
    assert!(empty_page.is_empty());
    assert_eq!(next_offset, ids[3]);
}

#[test]
fn test_list_points_with_zero_limit_returns_none() {
    let storage = create_test_storage();

    assert_eq!(storage.list_points(Uuid::nil(), 0).unwrap(), None);
}

#[test]
fn test_get_point_returns_all_fields() {
    let storage = create_test_storage();
    let id = Uuid::new_v4();
    let sparse_vector = SparseVector {
        token_ids: vec![1],
        weights: vec![2.0],
    };

    storage
        .insert_point(
            id,
            Some(vec![0.1, 0.2]),
            Some(sparse_vector.clone()),
            Some(test_payload("both")),
        )
        .unwrap();

    let point = storage.get_point(id).unwrap().unwrap();
    assert_eq!(point.dense_vector, Some(vec![0.1, 0.2]));
    assert_eq!(point.sparse_vector, Some(sparse_vector));
    assert_eq!(point.payload, Some(test_payload("both")));
    assert_eq!(storage.get_point(Uuid::new_v4()).unwrap(), None);
}

#[test]
fn test_create_and_restore_checkpoint() {
    let mut storage = create_test_storage();
    let temp_dir: TempDir = tempdir().unwrap();
    let id_before_checkpoint = Uuid::new_v4();
    let id_after_checkpoint = Uuid::new_v4();

    storage
        .insert_point(
            id_before_checkpoint,
            Some(vec![0.1, 0.2, 0.3]),
            None,
            Some(test_payload("before")),
        )
        .unwrap();
    let checkpoint = storage.checkpoint_at(temp_dir.path()).unwrap();

    storage
        .insert_point(
            id_after_checkpoint,
            Some(vec![0.4, 0.5, 0.6]),
            None,
            Some(test_payload("after")),
        )
        .unwrap();

    storage.restore_checkpoint(&checkpoint).unwrap();

    assert!(storage.contains_point(id_before_checkpoint).unwrap());
    assert!(!storage.contains_point(id_after_checkpoint).unwrap());
    assert_eq!(
        storage
            .get_point(id_before_checkpoint)
            .unwrap()
            .and_then(|point| point.payload),
        Some(test_payload("before"))
    );
}

#[test]
fn test_checkpoint_writes_header() {
    let storage = create_test_storage();
    let temp_dir = tempdir().unwrap();
    let id = Uuid::new_v4();

    storage
        .insert_point(
            id,
            Some(vec![0.1, 0.2, 0.3]),
            None,
            Some(test_payload("point")),
        )
        .unwrap();
    let checkpoint = storage.checkpoint_at(temp_dir.path()).unwrap();

    let mut file = File::open(checkpoint.path).unwrap();
    let mut magic = [0u8; INMEMORY_CHECKPOINT_MAGIC.len()];
    file.read_exact(&mut magic).unwrap();
    assert_eq!(&magic, INMEMORY_CHECKPOINT_MAGIC);

    let mut version_bytes = [0u8; size_of::<u16>()];
    file.read_exact(&mut version_bytes).unwrap();
    assert_eq!(
        u16::from_le_bytes(version_bytes),
        INMEMORY_CHECKPOINT_VERSION
    );

    let mut count_bytes = [0u8; size_of::<u64>()];
    file.read_exact(&mut count_bytes).unwrap();
    assert_eq!(u64::from_le_bytes(count_bytes), 1);
}

#[test]
fn test_restore_rejects_invalid_checkpoint_magic() {
    let mut storage = create_test_storage();
    let temp_dir = tempdir().unwrap();
    let checkpoint_path = temp_dir.path().join("inmemory-invalid.bin");
    let mut file = File::create(&checkpoint_path).unwrap();
    file.write_all(b"BADMAGIC").unwrap();
    file.write_all(&INMEMORY_CHECKPOINT_VERSION.to_le_bytes())
        .unwrap();
    file.write_all(&0u64.to_le_bytes()).unwrap();

    let checkpoint = StorageCheckpoint {
        path: checkpoint_path,
        storage_type: StorageType::InMemory,
    };

    let error = storage.restore_checkpoint(&checkpoint).unwrap_err();
    assert!(matches!(error, StorageError::InMemoryCheckpoint { .. }));
}

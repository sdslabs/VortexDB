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

    storage.insert_point(id, vector.clone(), payload).unwrap();

    assert_eq!(storage.get_vector(id).unwrap(), vector);
}

#[test]
fn test_insert_and_get_payload() {
    let storage = create_test_storage();
    let id = Uuid::new_v4();
    let payload = Some(test_payload("Test"));

    storage.insert_point(id, None, payload.clone()).unwrap();

    assert_eq!(storage.get_payload(id).unwrap(), payload);
}

#[test]
fn test_contains_and_delete_point() {
    let storage = create_test_storage();
    let id = Uuid::new_v4();

    assert!(!storage.contains_point(id).unwrap());

    storage
        .insert_point(id, Some(vec![0.4, 0.5, 0.6]), Some(test_payload("Test")))
        .unwrap();
    assert!(storage.contains_point(id).unwrap());

    storage.delete_point(id).unwrap();
    assert!(!storage.contains_point(id).unwrap());
    assert_eq!(storage.get_vector(id).unwrap(), None);
    assert_eq!(storage.get_payload(id).unwrap(), None);
}

#[test]
fn test_list_vectors_respects_offset_limit_and_skips_payload_only_points() {
    let storage = create_test_storage();
    let ids = [
        Uuid::from_u128(1),
        Uuid::from_u128(2),
        Uuid::from_u128(3),
        Uuid::from_u128(4),
    ];

    storage
        .insert_point(ids[0], Some(vec![1.0, 1.1]), Some(test_payload("one")))
        .unwrap();
    storage
        .insert_point(ids[1], None, Some(test_payload("payload-only")))
        .unwrap();
    storage
        .insert_point(ids[2], Some(vec![3.0, 3.1]), Some(test_payload("three")))
        .unwrap();
    storage
        .insert_point(ids[3], Some(vec![4.0, 4.1]), Some(test_payload("four")))
        .unwrap();

    let (first_page, next_offset) = storage.list_vectors(Uuid::nil(), 2).unwrap().unwrap();
    assert_eq!(
        first_page,
        vec![(ids[0], vec![1.0, 1.1]), (ids[2], vec![3.0, 3.1])]
    );
    assert_eq!(next_offset, ids[2]);

    let (second_page, next_offset) = storage.list_vectors(next_offset, 2).unwrap().unwrap();
    assert_eq!(second_page, vec![(ids[3], vec![4.0, 4.1])]);
    assert_eq!(next_offset, ids[3]);
}

#[test]
fn test_list_vectors_with_zero_limit_returns_none() {
    let storage = create_test_storage();

    assert_eq!(storage.list_vectors(Uuid::nil(), 0).unwrap(), None);
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
            Some(test_payload("before")),
        )
        .unwrap();
    let checkpoint = storage.checkpoint_at(temp_dir.path()).unwrap();

    storage
        .insert_point(
            id_after_checkpoint,
            Some(vec![0.4, 0.5, 0.6]),
            Some(test_payload("after")),
        )
        .unwrap();

    storage.restore_checkpoint(&checkpoint).unwrap();

    assert!(storage.contains_point(id_before_checkpoint).unwrap());
    assert!(!storage.contains_point(id_after_checkpoint).unwrap());
    assert_eq!(
        storage.get_payload(id_before_checkpoint).unwrap(),
        Some(test_payload("before"))
    );
}

#[test]
fn test_checkpoint_writes_header() {
    let storage = create_test_storage();
    let temp_dir = tempdir().unwrap();
    let id = Uuid::new_v4();

    storage
        .insert_point(id, Some(vec![0.1, 0.2, 0.3]), Some(test_payload("point")))
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

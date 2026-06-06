use crate::StorageType;
use crate::error::StorageError;
use crate::{StorageEngine, VectorPage, checkpoint::StorageCheckpoint};
use bincode::{deserialize_from, serialize_into};
use defs::{DenseVector, Payload, Point, PointId};
use std::collections::BTreeMap;
use std::fs::File;
use std::ops::Bound::{Excluded, Unbounded};
use std::path::Path;
use std::sync::RwLock;

pub const INMEMORY_CHECKPOINT_FILENAME_MARKER: &str = "inmemory";
const INMEMORY_CHECKPOINT_EXTENSION: &str = "bin";

pub struct MemoryStorage {
    points: RwLock<BTreeMap<PointId, Point>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        MemoryStorage {
            points: RwLock::new(BTreeMap::new()),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageEngine for MemoryStorage {
    fn insert_point(
        &self,
        id: PointId,
        vector: Option<DenseVector>,
        payload: Option<Payload>,
    ) -> Result<(), StorageError> {
        let mut points = self
            .points
            .write()
            .map_err(|_| StorageError::InMemoryLock {})?;
        points.insert(
            id,
            Point {
                id,
                vector,
                payload,
            },
        );
        Ok(())
    }
    fn contains_point(&self, id: PointId) -> Result<bool, StorageError> {
        let points = self
            .points
            .read()
            .map_err(|_| StorageError::InMemoryLock {})?;
        Ok(points.contains_key(&id))
    }
    fn delete_point(&self, id: PointId) -> Result<(), StorageError> {
        let mut points = self
            .points
            .write()
            .map_err(|_| StorageError::InMemoryLock {})?;
        points.remove(&id);
        Ok(())
    }
    fn get_payload(&self, id: PointId) -> Result<Option<Payload>, StorageError> {
        let points = self
            .points
            .read()
            .map_err(|_| StorageError::InMemoryLock {})?;
        Ok(points.get(&id).and_then(|point| point.payload.clone()))
    }
    fn get_vector(&self, id: PointId) -> Result<Option<DenseVector>, StorageError> {
        let points = self
            .points
            .read()
            .map_err(|_| StorageError::InMemoryLock {})?;
        Ok(points.get(&id).and_then(|point| point.vector.clone()))
    }
    fn list_vectors(
        &self,
        offset: PointId,
        limit: usize,
    ) -> Result<Option<VectorPage>, StorageError> {
        if limit < 1 {
            return Ok(None);
        }

        let points = self
            .points
            .read()
            .map_err(|_| StorageError::InMemoryLock {})?;
        let mut result = Vec::with_capacity(limit);
        let mut last_id = offset;

        for (id, point) in points.range((Excluded(offset), Unbounded)) {
            if let Some(vector) = &point.vector {
                last_id = *id;
                result.push((*id, vector.clone()));
                if result.len() == limit {
                    break;
                }
            }
        }

        Ok(Some((result, last_id)))
    }

    fn checkpoint_at(&self, path: &Path) -> Result<StorageCheckpoint, StorageError> {
        let checkpoint_filename = format!(
            "{}-{}.{}",
            INMEMORY_CHECKPOINT_FILENAME_MARKER,
            uuid::Uuid::new_v4(),
            INMEMORY_CHECKPOINT_EXTENSION
        );
        let checkpoint_path = path.join(checkpoint_filename);
        let file = File::create(&checkpoint_path).map_err(|source| {
            StorageError::InMemoryCheckpointIo {
                msg: "Couldn't create in-memory checkpoint".to_string(),
                source,
            }
        })?;
        let points = self
            .points
            .read()
            .map_err(|_| StorageError::InMemoryLock {})?;
        serialize_into(file, &*points).map_err(|source| StorageError::Serialization {
            id: PointId::nil(),
            source,
        })?;

        Ok(StorageCheckpoint {
            path: checkpoint_path,
            storage_type: StorageType::InMemory,
        })
    }

    fn restore_checkpoint(&mut self, checkpoint: &StorageCheckpoint) -> Result<(), StorageError> {
        if checkpoint.storage_type != StorageType::InMemory {
            return Err(StorageError::InMemoryCheckpoint {
                msg: "Invalid storage type".to_string(),
            });
        }

        let checkpoint_filename = checkpoint
            .path
            .file_name()
            .ok_or_else(|| StorageError::InMemoryCheckpoint {
                msg: "Could not read checkpoint filename".to_string(),
            })?
            .to_str()
            .ok_or_else(|| StorageError::InMemoryCheckpoint {
                msg: "Checkpoint filename is not valid UTF-8".to_string(),
            })?;
        if !checkpoint_filename.starts_with(INMEMORY_CHECKPOINT_FILENAME_MARKER)
            || checkpoint.path.extension().and_then(|ext| ext.to_str())
                != Some(INMEMORY_CHECKPOINT_EXTENSION)
        {
            return Err(StorageError::InMemoryCheckpoint {
                msg: "Invalid file name".to_string(),
            });
        }

        let file =
            File::open(&checkpoint.path).map_err(|source| StorageError::InMemoryCheckpointIo {
                msg: "Couldn't open in-memory checkpoint".to_string(),
                source,
            })?;
        let restored_points: BTreeMap<PointId, Point> =
            deserialize_from(file).map_err(|source| StorageError::Deserialization {
                id: PointId::nil(),
                source,
            })?;
        let mut points = self
            .points
            .write()
            .map_err(|_| StorageError::InMemoryLock {})?;
        *points = restored_points;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use defs::ContentType;
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
}

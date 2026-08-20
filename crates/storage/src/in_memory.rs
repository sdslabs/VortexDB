use crate::StorageType;
use crate::error::StorageError;
use crate::{StorageEngine, VectorPage, checkpoint::StorageCheckpoint};
use bincode::{deserialize_from, serialize_into};
use defs::{DenseVector, Payload, Point, PointId};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::ops::Bound::{Excluded, Unbounded};
use std::path::Path;
use std::sync::RwLock;

mod constants;
pub use constants::INMEMORY_CHECKPOINT_FILENAME_MARKER;
use constants::{
    INMEMORY_CHECKPOINT_EXTENSION, INMEMORY_CHECKPOINT_MAGIC, INMEMORY_CHECKPOINT_VERSION,
};

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
        let point_snapshot: Vec<Point> = {
            let points = self
                .points
                .read()
                .map_err(|_| StorageError::InMemoryLock {})?;
            points.values().cloned().collect()
        };

        let mut writer = BufWriter::new(file);
        writer
            .write_all(INMEMORY_CHECKPOINT_MAGIC)
            .and_then(|_| writer.write_all(&INMEMORY_CHECKPOINT_VERSION.to_le_bytes()))
            .and_then(|_| writer.write_all(&(point_snapshot.len() as u64).to_le_bytes()))
            .map_err(|source| StorageError::InMemoryCheckpointIo {
                msg: "Couldn't write in-memory checkpoint header".to_string(),
                source,
            })?;

        for point in &point_snapshot {
            serialize_into(&mut writer, point).map_err(|source| StorageError::Serialization {
                id: point.id,
                source,
            })?;
        }

        writer
            .flush()
            .map_err(|source| StorageError::InMemoryCheckpointIo {
                msg: "Couldn't flush in-memory checkpoint".to_string(),
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
        let mut reader = BufReader::new(file);
        let mut magic = [0u8; INMEMORY_CHECKPOINT_MAGIC.len()];
        reader
            .read_exact(&mut magic)
            .map_err(|source| StorageError::InMemoryCheckpointIo {
                msg: "Couldn't read in-memory checkpoint magic".to_string(),
                source,
            })?;
        if &magic != INMEMORY_CHECKPOINT_MAGIC {
            return Err(StorageError::InMemoryCheckpoint {
                msg: "Invalid checkpoint magic".to_string(),
            });
        }

        let mut version_bytes = [0u8; size_of::<u16>()];
        reader.read_exact(&mut version_bytes).map_err(|source| {
            StorageError::InMemoryCheckpointIo {
                msg: "Couldn't read in-memory checkpoint version".to_string(),
                source,
            }
        })?;
        let version = u16::from_le_bytes(version_bytes);
        if version != INMEMORY_CHECKPOINT_VERSION {
            return Err(StorageError::InMemoryCheckpoint {
                msg: format!("Unsupported checkpoint version: {version}"),
            });
        }

        let mut count_bytes = [0u8; size_of::<u64>()];
        reader.read_exact(&mut count_bytes).map_err(|source| {
            StorageError::InMemoryCheckpointIo {
                msg: "Couldn't read in-memory checkpoint point count".to_string(),
                source,
            }
        })?;
        let point_count = u64::from_le_bytes(count_bytes);
        let mut restored_points = BTreeMap::new();
        for _ in 0..point_count {
            let point: Point =
                deserialize_from(&mut reader).map_err(|source| StorageError::Deserialization {
                    id: PointId::nil(),
                    source,
                })?;
            restored_points.insert(point.id, point);
        }

        let mut points = self
            .points
            .write()
            .map_err(|_| StorageError::InMemoryLock {})?;
        *points = restored_points;
        Ok(())
    }
}

#[cfg(test)]
mod tests;

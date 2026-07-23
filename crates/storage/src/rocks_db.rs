// Rewrite needed

use crate::checkpoint::StorageCheckpoint;
use crate::error::{
    self, RocksDbCheckpointIoSnafu, RocksDbCheckpointMsgSnafu, RocksDbCheckpointSnafu,
    RocksDbFlushSnafu, RocksDbInitializationSnafu, StorageError,
};
use crate::{StorageEngine, StorageType, VectorPage};
use bincode::{deserialize, serialize};
use defs::{DenseVector, Payload, Point, PointId};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use rocksdb::{DB, Error, Options};
use snafu::{OptionExt, ResultExt};
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use tempfile::tempdir;

//TODO: Implement RocksDbStorage with necessary fields and implementations
//TODO: Optimize the basic design
pub struct RocksDbStorage {
    pub path: PathBuf,
    pub db: Option<DB>,
}

pub enum RocksDBStorageError {
    RocksDBError(Error),
}

pub const ROCKSDB_CHECKPOINT_FILENAME_MARKER: &str = "rocksdb";

impl RocksDbStorage {
    // Creates new db or switches to existing db
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let converted_path = path.into();
        let db = Self::initialize_db(&converted_path)?;

        Ok(RocksDbStorage {
            path: converted_path,
            db: Some(db),
        })
    }

    fn initialize_db(path: &Path) -> Result<DB, StorageError> {
        // Initialize a db at the given location
        let mut options = Options::default();

        // Optimize RocksDB
        options.increase_parallelism(12);
        options.optimize_level_style_compaction(512 * 1024 * 1024);

        options.create_if_missing(true);

        let db = DB::open(&options, path).map_err(|e| StorageError::RocksDbOpen {
            path: path.to_string_lossy().into_owned(),
            source: e,
        })?;
        Ok(db)
    }

    pub fn get_current_path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl StorageEngine for RocksDbStorage {
    fn insert_point(
        &self,
        id: PointId,
        vector: Option<DenseVector>,
        payload: Option<Payload>,
    ) -> Result<(), StorageError> {
        let key = id.to_string();
        let point = Point {
            id,
            vector,
            payload,
        };
        let value = serialize(&point).context(error::SerializationSnafu { id })?;

        self.db
            .as_ref()
            .context(error::RocksDbInitializationSnafu {})?
            .put(key.as_bytes(), value.as_slice())
            .context(error::RocksDbWriteSnafu { id })?;

        Ok(())
    }

    fn contains_point(&self, id: PointId) -> Result<bool, StorageError> {
        // Efficient lookup inspired from https://github.com/facebook/rocksdb/issues/11586#issuecomment-1890429488
        let key = id.to_string();
        if self
            .db
            .as_ref()
            .context(error::RocksDbInitializationSnafu {})?
            .key_may_exist(key.clone())
        {
            let key_exist = self
                .db
                .as_ref()
                .context(error::RocksDbInitializationSnafu {})?
                .get(key)
                .context(error::RocksDbReadSnafu { id })?
                .is_some();
            Ok(key_exist)
        } else {
            Ok(false)
        }
    }

    fn delete_point(&self, id: PointId) -> Result<(), StorageError> {
        let key = id.to_string();
        self.db
            .as_ref()
            .context(error::RocksDbInitializationSnafu {})?
            .delete(key)
            .context(error::RocksDbDeleteSnafu { id })?;

        Ok(())
    }

    fn get_payload(&self, id: PointId) -> Result<Option<Payload>, StorageError> {
        let key = id.to_string();
        let Some(value_serialized) = self
            .db
            .as_ref()
            .ok_or(StorageError::RocksDbInitialization {})?
            .get(key)
            .context(error::RocksDbReadSnafu { id })?
        else {
            return Ok(None); // This should not return error but rather give None
        };

        let value =
            deserialize::<Point>(&value_serialized).context(error::DeserializationSnafu { id })?;

        Ok(value.payload)
    }

    fn get_vector(&self, id: PointId) -> Result<Option<DenseVector>, StorageError> {
        let key = id.to_string();
        let Some(value_serialized) = self
            .db
            .as_ref()
            .ok_or(StorageError::RocksDbInitialization {})?
            .get(key)
            .context(error::RocksDbReadSnafu { id })?
        else {
            return Ok(None); // This should not return error but rather give None
        };

        let value =
            deserialize::<Point>(&value_serialized).context(error::DeserializationSnafu { id })?;

        Ok(value.vector)
    }

    fn list_vectors(
        &self,
        offset: PointId,
        limit: usize,
    ) -> Result<Option<VectorPage>, StorageError> {
        if limit < 1 {
            return Ok(None);
        }

        let mut result = Vec::with_capacity(limit);
        let iter = self
            .db
            .as_ref()
            .context(error::RocksDbInitializationSnafu {})?
            .iterator(rocksdb::IteratorMode::From(
                offset.to_string().as_bytes(),
                rocksdb::Direction::Forward,
            ));
        let mut last_id = offset;

        for item in iter {
            let (_, v) = item.context(error::RocksDbIterationSnafu)?;
            let point: Point =
                deserialize(&v).context(error::DeserializationSnafu { id: offset })?;

            let id = point.id;

            if id <= offset {
                continue;
            }

            if let Some(vec) = point.vector {
                last_id = id;
                result.push((id, vec));
                if result.len() == limit {
                    break;
                }
            }
        }
        Ok(Some((result, last_id)))
    }

    fn checkpoint_at(&self, path: &Path) -> Result<StorageCheckpoint, StorageError> {
        // flush db first for durability
        self.db
            .as_ref()
            .ok_or(StorageError::RocksDbInitialization {})?
            .flush()
            .context(RocksDbFlushSnafu)?;

        // filename is rocksdb-{uuid}.tar.gz
        let checkpoint_filename = format!(
            "{}-{}.tar.gz",
            ROCKSDB_CHECKPOINT_FILENAME_MARKER,
            uuid::Uuid::new_v4()
        );
        let checkpoint_path = path.join(checkpoint_filename);

        let temp_dir_parent = tempdir().unwrap();
        let temp_dir = temp_dir_parent.path().join("checkpoint");

        let db_ref = self
            .db
            .as_ref()
            .ok_or(StorageError::RocksDbInitialization {})?;

        let checkpoint =
            rocksdb::checkpoint::Checkpoint::new(db_ref).context(RocksDbCheckpointSnafu)?;
        checkpoint
            .create_checkpoint(temp_dir.clone())
            .context(RocksDbCheckpointSnafu)?;

        // compress the checkpoint into an archive
        let tar_gz =
            File::create(checkpoint_path.clone()).with_context(|e| RocksDbCheckpointIoSnafu {
                msg: format!("Couldn't create tar.gz archive: {}", e),
            })?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut archive = Builder::new(enc);

        archive
            .append_dir_all("", temp_dir)
            .with_context(|e| RocksDbCheckpointIoSnafu {
                msg: format!("Couldn't append directory to archive: {}", e),
            })?;

        let enc = archive
            .into_inner()
            .with_context(|e| RocksDbCheckpointIoSnafu {
                msg: format!("Couldn't compress tar.gz archive: {}", e),
            })?;

        enc.finish().with_context(|e| RocksDbCheckpointIoSnafu {
            msg: format!("Couldn't compress tar.gz archive: {}", e),
        })?;

        Ok(StorageCheckpoint {
            path: checkpoint_path,
            storage_type: crate::StorageType::RocksDb,
        })
    }

    fn restore_checkpoint(&mut self, checkpoint: &StorageCheckpoint) -> Result<(), StorageError> {
        // enforce storage type
        if checkpoint.storage_type != StorageType::RocksDb {
            return Err(StorageError::RocksDbCheckpointMsg {
                msg: "Invalid storage type".to_string(),
            });
        }
        // enforce filename marker - should have been enforced during StoraegCheckpoint::open anyway
        let checkpoint_filename = checkpoint
            .path
            .file_name()
            .ok_or_else(|| StorageError::RocksDbCheckpointMsg {
                msg: "Could not read checkpoint filename".to_string(),
            })?
            .to_str()
            .ok_or_else(|| StorageError::RocksDbCheckpointMsg {
                msg: "Checkpoint filename is not valid UTF-8".to_string(),
            })?;
        if !checkpoint_filename.ends_with(".tar.gz")
            || !checkpoint_filename.starts_with(ROCKSDB_CHECKPOINT_FILENAME_MARKER)
        {
            return RocksDbCheckpointMsgSnafu {
                msg: "Invalid file name".to_string(),
            }
            .fail();
        }

        let tar_gz = File::open(&checkpoint.path).with_context(|e| RocksDbCheckpointIoSnafu {
            msg: format!("Couldn't open checkpoint file: {}", e),
        })?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        // remove existing stuff in data path
        self.db
            .as_ref()
            .context(RocksDbInitializationSnafu)?
            .cancel_all_background_work(true);
        // drop db early
        self.db = None;

        std::fs::remove_dir_all(&self.path).with_context(|e| RocksDbCheckpointIoSnafu {
            msg: format!("Couldn't remove existing data: {}", e),
        })?;

        // create new data path
        std::fs::create_dir_all(&self.path).with_context(|e| RocksDbCheckpointIoSnafu {
            msg: format!("Couldn't create data path: {}", e),
        })?;

        archive
            .unpack(&self.path)
            .with_context(|e| RocksDbCheckpointIoSnafu {
                msg: format!("Couldn't unpack tar.gz archive: {}", e),
            })?;

        // reinitialize db
        self.db = Some(Self::initialize_db(&self.path)?);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
}

pub mod constants;
pub mod engine;
pub mod manifest;
pub mod metadata;
pub mod registry;
mod util;

use crate::{
    constants::{MANIFEST_FILE, SNAPSHOT_PARSER_VER},
    manifest::Manifest,
    util::{compress_archive, save_index_metadata, save_topology},
};

use chrono::{DateTime, Local};
use defs::DbError;
use flate2::read::GzDecoder;
use index::{
    IndexSnapshot, IndexType, VectorIndex, flat::index::FlatIndex, hnsw::HnswIndex,
    kd_tree::index::KDTree,
};
use semver::Version;
use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::SystemTime,
};
use storage::{
    StorageEngine, StorageType, checkpoint::StorageCheckpoint, in_memory::MemoryStorage,
    rocks_db::RocksDbStorage,
};
use tar::Archive;
use tempfile::tempdir;
use uuid::Uuid;

type VectorDbRestore = (Arc<dyn StorageEngine>, Arc<RwLock<dyn VectorIndex>>, usize);

pub struct Snapshot {
    pub id: Uuid,
    pub date: SystemTime,
    pub sem_ver: Version,
    pub index_snapshot: IndexSnapshot,
    pub storage_snapshot: StorageCheckpoint,
    pub dimensions: usize,
}

impl Snapshot {
    pub fn new(
        index_snapshot: IndexSnapshot,
        storage_snapshot: StorageCheckpoint,
        dimensions: usize,
    ) -> Result<Snapshot, DbError> {
        let id = Uuid::new_v4();
        let date = SystemTime::now();

        Ok(Snapshot {
            id,
            date,
            sem_ver: SNAPSHOT_PARSER_VER,
            index_snapshot,
            storage_snapshot,
            dimensions,
        })
    }

    pub fn save(&self, dir_path: &Path) -> Result<PathBuf, DbError> {
        if !dir_path.is_dir() {
            return Err(DbError::SnapshotError(format!(
                "Invalid path: {}",
                dir_path.display()
            )));
        }

        let temp_dir = tempdir().map_err(|e| DbError::SnapshotError(e.to_string()))?;

        // save index snapshots
        let index_metadata_path = save_index_metadata(
            temp_dir.path(),
            self.id,
            &self.index_snapshot.metadata_b,
            &self.index_snapshot.magic,
        )?;

        let topology_path = save_topology(
            temp_dir.path(),
            self.id,
            &self.index_snapshot.topology_b,
            &self.index_snapshot.magic,
        )?;

        // take checksums
        let index_metadata_checksum = util::sha256_digest(&index_metadata_path)
            .map_err(|e| DbError::SnapshotError(e.to_string()))?;
        let index_topo_checksum = util::sha256_digest(&topology_path)
            .map_err(|e| DbError::SnapshotError(e.to_string()))?;
        let storage_checkpoint_checksum = util::sha256_digest(&self.storage_snapshot.path)
            .map_err(|e| DbError::SnapshotError(e.to_string()))?;

        let dt_now_local: DateTime<Local> = self.date.into();

        // need this for manifest
        let storage_checkpoint_filename = self
            .storage_snapshot
            .path
            .file_name()
            .ok_or(DbError::SnapshotError(
                "Storage checkpoint was not properly made".to_string(),
            ))?
            .to_str()
            .ok_or(DbError::SnapshotError(
                "Storage checkpoint filename is not valid UTF-8".to_string(),
            ))?
            .to_string();

        // create manifest file
        let manifest = Manifest {
            id: self.id,
            date: dt_now_local.timestamp(),
            sem_ver: constants::SNAPSHOT_PARSER_VER.to_string(),
            index_metadata_checksum,
            index_topo_checksum,
            storage_checkpoint_checksum,
            storage_type: self.storage_snapshot.storage_type,
            index_type: self.index_snapshot.index_type,
            dimensions: self.dimensions,
            storage_checkpoint_filename,
        };

        let manifest_path = manifest
            .save(temp_dir.path())
            .map_err(|e| DbError::SnapshotError(e.to_string()))?;

        let tar_filename = format!(
            "{}.tar.gz",
            metadata::Metadata::new(
                self.id,
                self.date,
                index_metadata_path.clone(),
                constants::SNAPSHOT_PARSER_VER
            )
        );
        let tar_gz_path = dir_path.join(tar_filename);

        compress_archive(
            &tar_gz_path,
            &[
                &index_metadata_path,
                &topology_path,
                &self.storage_snapshot.path,
                &manifest_path,
            ],
        )
        .map_err(|e| DbError::SnapshotError(e.to_string()))?;
        Ok(tar_gz_path.to_path_buf())
    }

    pub fn load(path: &Path, storage_data_path: &Path) -> Result<VectorDbRestore, DbError> {
        let tar_gz = File::open(path)
            .map_err(|e| DbError::SnapshotError(format!("Couldn't open snapshot: {}", e)))?;

        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        let snapshot_filename = path.file_name().ok_or(DbError::SnapshotError(
            "Invalid snapshot filename".to_string(),
        ))?;
        let temp_dir = std::env::temp_dir().join(snapshot_filename);

        // remove any existing data
        if temp_dir.exists() && !temp_dir.is_dir() {
            std::fs::remove_file(temp_dir.clone()).map_err(|e| {
                DbError::SnapshotError(format!("Couldn't remove existing file: {}", e))
            })?;
        } else if temp_dir.is_dir() {
            std::fs::remove_dir_all(temp_dir.clone()).map_err(|e| {
                DbError::SnapshotError(format!("Couldn't remove existing directory: {}", e))
            })?;
        }

        std::fs::create_dir(temp_dir.clone()).map_err(|e| {
            DbError::SnapshotError(format!("Couldn't create temporary directory: {}", e))
        })?;

        archive
            .unpack(temp_dir.clone())
            .map_err(|e| DbError::SnapshotError(format!("Couldn't unpack archive: {}", e)))?;

        // read manifest and validate
        let manifest_path = temp_dir.join(MANIFEST_FILE);
        if !manifest_path.is_file() {
            return Err(DbError::SnapshotError(
                "Manifest file not found".to_string(),
            ));
        }

        let manifest = Manifest::load(&manifest_path)
            .map_err(|e| DbError::SnapshotError(format!("Couldn't load manifest: {}", e)))?;

        if manifest.sem_ver != SNAPSHOT_PARSER_VER.to_string() {
            return Err(DbError::SnapshotError(
                "Incompatible snapshot version".to_string(),
            ));
        }

        let mut storage_engine: Box<dyn StorageEngine> = match manifest.storage_type {
            StorageType::InMemory => Box::new(MemoryStorage::new()),
            StorageType::RocksDb => Box::new(
                RocksDbStorage::new(storage_data_path)
                    .map_err(|e| DbError::StorageError(format!("Could not open storage: {e}")))?,
            ),
        };

        let id = manifest.id;
        let index_metadata_path = temp_dir.join(util::metadata_filename(&id));
        let topology_path = temp_dir.join(util::topology_filename(&id));
        let storage_checkpoint_path = temp_dir.join(manifest.storage_checkpoint_filename);

        if !index_metadata_path.exists()
            || !topology_path.exists()
            || !storage_checkpoint_path.exists()
        {
            return Err(DbError::SnapshotError(format!(
                "Missing snapshot files {} , {}, {}",
                index_metadata_path.display(),
                topology_path.display(),
                storage_checkpoint_path.display()
            )));
        }

        // match checksums
        if util::sha256_digest(&index_metadata_path).map_err(|_| {
            DbError::SnapshotError("Could not calculate index metadata hash".to_string())
        })? != manifest.index_metadata_checksum
        {
            return Err(DbError::SnapshotError(
                "Index metadata hash mismatch".to_string(),
            ));
        }
        if util::sha256_digest(&topology_path)
            .map_err(|_| DbError::SnapshotError("Could not calculate topology hash".to_string()))?
            != manifest.index_topo_checksum
        {
            return Err(DbError::SnapshotError("Topology hash mismatch".to_string()));
        }
        if util::sha256_digest(&storage_checkpoint_path).map_err(|_| {
            DbError::SnapshotError("Could not calculate storage checkpoint hash".to_string())
        })? != manifest.storage_checkpoint_checksum
        {
            return Err(DbError::SnapshotError(
                "Storage checkpoint hash mismatch".to_string(),
            ));
        }

        let (mgmeta, meta_bytes) = util::read_index_metadata(&index_metadata_path)
            .map_err(|_| DbError::SnapshotError("Could not read metadata".to_string()))?;
        let (mgtopo, topo_bytes) = util::read_index_topology(&topology_path)
            .map_err(|_| DbError::SnapshotError("Could not read topology".to_string()))?;

        if mgtopo != mgmeta {
            return Err(DbError::InvalidMagicBytes(
                "Magic bytes don't match".to_string(),
            ));
        }

        // validates if manifest storage type matches that in the filename of storage checkpoint
        let storage_checkpoint = StorageCheckpoint::open(storage_checkpoint_path.as_path())?;
        if storage_checkpoint.storage_type != manifest.storage_type {
            return Err(DbError::SnapshotError(
                "Storage type mismatch from manifest and checkpoint".to_string(),
            ));
        }

        storage_engine
            .restore_checkpoint(&storage_checkpoint)
            .map_err(|e| {
                DbError::StorageCheckpointError(format!("Could not restore checkpoint: {e}"))
            })?;

        let index_snapshot = IndexSnapshot {
            index_type: manifest.index_type,
            magic: mgmeta,
            metadata_b: meta_bytes,
            topology_b: topo_bytes,
        };

        // dynamic dispatch based on index type
        let vector_index: Arc<RwLock<dyn VectorIndex>> = match manifest.index_type {
            IndexType::Flat => Arc::new(RwLock::new(FlatIndex::deserialize(&index_snapshot)?)),
            IndexType::KDTree => Arc::new(RwLock::new(KDTree::deserialize(&index_snapshot)?)),
            IndexType::HNSW => Arc::new(RwLock::new(HnswIndex::deserialize(&index_snapshot)?)),
        };

        vector_index
            .write()
            .map_err(|_| DbError::LockError)?
            .populate_vectors(&*storage_engine)?;

        Ok((storage_engine.into(), vector_index, manifest.dimensions))
    }
}

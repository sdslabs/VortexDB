use std::path::{Path, PathBuf};

use defs::DbError;
pub mod constants;
pub mod local;
use crate::{VectorDbRestore, metadata::Metadata};

pub type SnapshotMetaPage = Vec<Metadata>;

pub const INFINITY_LIMIT: usize = 100000;
pub const NO_OFFSET: usize = 0;

pub trait SnapshotRegistry: Send + Sync {
    fn add_snapshot(&mut self, snapshot_path: &Path) -> Result<Metadata, DbError>;

    fn list_snapshots(&mut self, limit: usize, offset: usize) -> Result<SnapshotMetaPage, DbError>;
    fn get_latest_snapshot(&mut self) -> Result<Metadata, DbError>;

    fn get_metadata(&mut self, small_id: String) -> Result<Metadata, DbError>;
    fn remove_snapshot(&mut self, small_id: String) -> Result<Metadata, DbError>;

    fn load(
        &mut self,
        small_id: String,
        storage_data_path: &Path,
    ) -> Result<VectorDbRestore, DbError>;
    fn dir(&self) -> PathBuf;

    // in the future this could be used to maybe move an old/stale snapshot to cold storage or to a remote registry
    fn mark_dead(&mut self, small_id: String) -> Result<Metadata, DbError>; // current behaviour is to call remove_snapshot;
    fn list_alive_snapshots(&mut self) -> Result<SnapshotMetaPage, DbError>; // current behaviour is to call list_snapshots;
}

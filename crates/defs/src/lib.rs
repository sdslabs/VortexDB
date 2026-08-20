pub mod error;
pub mod types;

// Without re-exports, users would need to write defs::types::SomeType instead of just defs::SomeType. Re-exports simplify the API by flattening the module hierarchy. The * means "everything public" from that module.
pub use error::*;
use std::path::{Path, PathBuf};
pub use types::*;

// hoisted trait so it can be used by the snapshots crate
pub trait SnapshottableDb: Send + Sync {
    fn create_snapshot(&self, dir_path: &Path) -> Result<PathBuf, DbError>;
}

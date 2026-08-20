use defs::Magic;

pub mod helpers;
pub mod index;
mod serialize;
pub mod types;

#[cfg(test)]
mod tests;

pub const KD_TREE_MAGIC_BYTES: Magic = [0x00, 0x01, 0x02, 0x00];

pub use index::{KDTree, KDTreeConfig};

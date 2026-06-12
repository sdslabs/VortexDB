// Referenced from HNSW (Malkov & Yashunin, 2018)
// https://arxiv.org/abs/1603.09320

pub mod index;
pub mod search;
pub mod serialize;
pub mod types;
use defs::Magic;
pub use index::{HnswConfig, HnswIndex};

pub const HNSW_MAGIC_BYTES: Magic = [0x02, 0x01, 0x03, 0x00];

#[cfg(test)]
mod tests;

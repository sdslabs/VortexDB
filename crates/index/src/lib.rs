use defs::{DbError, DenseVector, IndexedVector, Magic, PointId, Similarity};
pub use error::{IndexError, Result};

pub mod error;

use serde::{Deserialize, Serialize};
use storage::StorageEngine;
pub mod flat;
pub mod hnsw;
pub mod kd_tree;

pub trait VectorIndex: Send + Sync + SerializableIndex {
    fn insert(&mut self, vector: IndexedVector) -> Result<()>;

    // Returns true if point id existed and is deleted, else returns false
    fn delete(&mut self, point_id: PointId) -> Result<bool>;

    fn search(
        &self,
        query_vector: DenseVector,
        similarity: Similarity,
        k: usize,
    ) -> Result<Vec<PointId>>; // Return a Vec of ids of closest vectors (length max k)

    fn search_with_ef(
        &self,
        query_vector: DenseVector,
        similarity: Similarity,
        k: usize,
        _ef: Option<usize>,
    ) -> Result<Vec<PointId>> {
        self.search(query_vector, similarity, k)
    }

    fn insert_batch(&mut self, vectors: Vec<IndexedVector>) -> Result<()> {
        for v in vectors {
            self.insert(v)?;
        }
        Ok(())
    }
}

/// Distance function to get the distance between two vectors (taken from old version)
pub fn distance(a: &[f32], b: &[f32], dist_type: Similarity) -> f32 {
    assert_eq!(a.len(), b.len());
    match dist_type {
        Similarity::Euclidean => a
            .iter()
            .zip(b.iter())
            .map(|(&x, &y)| {
                let d = x - y;
                d * d
            })
            .sum::<f32>()
            .sqrt(),
        Similarity::Manhattan => a
            .iter()
            .zip(b.iter())
            .map(|(&x, &y)| (x - y).abs())
            .sum::<f32>(),
        Similarity::Hamming => a
            .iter()
            .zip(b.iter())
            .map(|(&x, &y)| if (x - y).abs() > 1e-8 { 1f32 } else { 0f32 })
            .sum::<f32>(),
        Similarity::Cosine => {
            let mut dot = 0.0f32;
            let mut norm_a = 0.0f32;
            let mut norm_b = 0.0f32;
            for (&x, &y) in a.iter().zip(b.iter()) {
                dot += x * y;
                norm_a += x * x;
                norm_b += y * y;
            }
            1.0 - dot / (norm_a.sqrt() * norm_b.sqrt())
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IndexType {
    Flat,
    KDTree,
    HNSW,
}

pub struct IndexSnapshot {
    pub index_type: IndexType,
    pub magic: Magic,
    pub topology_b: Vec<u8>,
    pub metadata_b: Vec<u8>,
}

pub trait SerializableIndex {
    fn serialize_topology(&self) -> Result<Vec<u8>, DbError>;
    fn serialize_metadata(&self) -> Result<Vec<u8>, DbError>;

    fn snapshot(&self) -> Result<IndexSnapshot, DbError>;

    fn populate_vectors(&mut self, storage: &dyn StorageEngine) -> Result<(), DbError>;
}

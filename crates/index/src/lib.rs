use defs::{DenseVector, IndexedVector, PointId, Similarity};
pub use error::{IndexError, Result};

pub mod error;
pub mod flat;
pub mod hnsw;
pub mod kd_tree;

pub trait VectorIndex: Send + Sync {
    fn insert(&mut self, vector: IndexedVector) -> Result<()>;

    // Returns true if point id existed and is deleted, else returns false
    fn delete(&mut self, point_id: PointId) -> Result<bool>;

    fn search(
        &self,
        query_vector: DenseVector,
        similarity: Similarity,
        k: usize,
    ) -> Result<Vec<PointId>>; // Return a Vec of ids of closest vectors (length max k)
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

#[derive(Debug, Clone, Copy)]
pub enum IndexType {
    Flat,
    KDTree,
    HNSW,
}

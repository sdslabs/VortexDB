use defs::{DbError, DenseVector, IndexedVector, PointId, Similarity};

pub mod flat;

pub trait VectorIndex: Send + Sync {
    fn insert(&mut self, vector: IndexedVector) -> Result<(), DbError>;

    // Returns true if point id existed and is deleted, else returns false
    fn delete(&mut self, point_id: PointId) -> Result<bool, DbError>;

    fn search(
        &self,
        query_vector: DenseVector,
        similarity: Similarity,
        k: usize,
    ) -> Result<Vec<PointId>, DbError>; // Return a Vec of ids of closest vectors (length max k)

    // fn build() -> Result<(), DbError>; move this to impl for dyn compatibility
}

/// Distance function to get the distance between two vectors (taken from old version)
pub fn distance(a: DenseVector, b: DenseVector, dist_type: Similarity) -> f32 {
    assert_eq!(a.len(), b.len());
    match dist_type {
        Similarity::Euclidean => {
            let score: Vec<f32> = a
                .iter()
                .zip(b.iter())
                .map(|(&x, &y)| (x - y) * (x - y))
                .collect();
            score.iter().sum::<f32>().sqrt()
        }
        Similarity::Manhattan => {
            let score: Vec<f32> = a
                .iter()
                .zip(b.iter())
                .map(|(&x, &y)| (x - y).abs())
                .collect();
            score.iter().sum::<f32>()
        }
        Similarity::Hamming => {
            let score: Vec<f32> = a
                .iter()
                .zip(b.iter())
                .map(|(&x, &y)| if (x - y).abs() > 1e-8 { 1f32 } else { 0f32 })
                .collect();
            score.iter().sum::<f32>()
        }
        Similarity::Cosine => {
            let p_score: Vec<f32> = a.iter().zip(b.iter()).map(|(&x, &y)| x * y).collect();
            let p = p_score.iter().sum::<f32>();
            let q_score: Vec<f32> = a.iter().map(|&n| n * n).collect();
            let q = q_score.iter().sum::<f32>().sqrt();
            let r_score: Vec<f32> = b.iter().map(|&n| n * n).collect();
            let r = r_score.iter().sum::<f32>().sqrt();
            1.0 - (p / (q * r))
        }
    }
}

pub enum IndexType {
    Flat,
    KDTree,
    HNSW,
}

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use uuid::Uuid;

pub type PointId = Uuid;

/// Type of vector element.
pub type Element = f32;
// pub type ElementHalf = f16; - Unstable https://github.com/rust-lang/rust/issues/116909
pub type ElementByte = u8;

// Dense Vector and Vector are considered same
// Sparse vector implementation not supported yet. Refer lib/sparse/src/common/sparse_vector.rs
pub type DenseVector = Vec<Element>;

pub enum StoredVector {
    Dense(DenseVector),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ContentType {
    Text,
    Image,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Payload {
    pub content_type: ContentType,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Point {
    pub id: PointId,
    pub vector: Option<DenseVector>,
    pub payload: Option<Payload>,
}

/// Struct which will be stored in the vector index
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct IndexedVector {
    pub id: PointId,
    pub vector: DenseVector,
}

#[derive(Deserialize, Copy, Clone)]
pub enum Similarity {
    Euclidean,
    Manhattan,
    Hamming,
    Cosine,
}

// Struct which stores the distance between a vector and query vector and implements ordering traits
#[derive(Copy, Clone)]
pub struct DistanceOrderedVector<'q> {
    // 'q : lifetime of query vector
    pub distance: f32,
    pub query_vector: &'q DenseVector,
    pub point_id: Option<PointId>,
}

impl<'q> Ord for DistanceOrderedVector<'q> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.total_cmp(&other.distance)
    }
}

impl<'q> PartialOrd for DistanceOrderedVector<'q> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'q> PartialEq for DistanceOrderedVector<'q> {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl<'q> Eq for DistanceOrderedVector<'q> {}

// Query Vector. Basically the type of query results that can be generated. Not implementing this but referencing here for furture reference
// #[derive(Debug, Clone)]
// pub enum QueryVector {
//     Nearest(VectorInternal),
//     RecommendBestScore(RecoQuery<VectorInternal>),
//     RecommendSumScores(RecoQuery<VectorInternal>),
//     Discovery(DiscoveryQuery<VectorInternal>),
//     Context(ContextQuery<VectorInternal>),
// }

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use uuid::Uuid;

use crate::SparseError;

pub type PointId = Uuid;

/// Type of vector element.
pub type Element = f32;
// pub type ElementHalf = f16; - Unstable https://github.com/rust-lang/rust/issues/116909
pub type ElementByte = u8;

pub type Dimension = usize;

pub type TokenID = u32;

pub type DenseVector = Vec<Element>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SparseVector {
    pub token_ids: Vec<TokenID>,
    pub weights: Vec<Element>,
}

impl SparseVector {
    pub fn validate(&self) -> Result<(), SparseError> {
        if self.token_ids.len() != self.weights.len() {
            return Err(SparseError::LengthMismatch {
                ids: self.token_ids.len(),
                weights: self.weights.len(),
            });
        }

        if self.token_ids.is_empty() {
            return Err(SparseError::Empty);
        }

        if let Some(i) = self.token_ids.windows(2).position(|w| w[0] >= w[1]) {
            return Err(SparseError::UnsortedTokenIds { index: i + 1 });
        }

        if let Some(i) = self.weights.iter().position(|w| !w.is_finite()) {
            return Err(SparseError::NonFiniteWeight { index: i });
        }

        if let Some(i) = self.weights.iter().position(|&w| w < 0.0) {
            return Err(SparseError::NegativeWeight { index: i });
        }

        Ok(())
    }
}

pub type Magic = [u8; 4];

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub enum ContentType {
    #[default]
    Text,
    Image,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Payload {
    pub content_type: ContentType,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PointInput {
    pub id: Option<PointId>,
    pub dense_vector: Option<DenseVector>,
    pub sparse_vector: Option<SparseVector>,
    pub payload: Option<Payload>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Point {
    pub id: PointId,
    pub dense_vector: Option<DenseVector>,
    pub sparse_vector: Option<SparseVector>,
    pub payload: Option<Payload>,
}

/// Struct which will be stored in the vector index
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct IndexedVector {
    pub id: PointId,
    pub vector: DenseVector,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
pub enum Similarity {
    Euclidean,
    Manhattan,
    Hamming,
    Cosine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInsertRequest {
    pub points: Vec<PointInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInsertResponse {
    pub inserted: usize,
    pub ids: Vec<PointId>,
}

// For batch search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSearchRequest {
    pub queries: Vec<SearchQuery>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DenseQuery {
    pub vector: DenseVector,
    pub similarity: Similarity,
    #[serde(default)]
    pub ef: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SearchHit {
    pub id: PointId,
    pub score: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum QueryVector {
    Dense(DenseQuery),
    Sparse(SparseVector),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchQuery {
    pub limit: usize,
    pub query: QueryVector,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSearchResponse {
    pub results: Vec<SearchResponse>,
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

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct OrdF32(pub f32);

impl OrdF32 {
    pub fn new(x: f32) -> Self {
        Self(x)
    }
    pub fn into_inner(self) -> f32 {
        self.0
    }
}

impl Eq for OrdF32 {}

impl Ord for OrdF32 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialOrd for OrdF32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<f32> for OrdF32 {
    fn from(x: f32) -> Self {
        Self(x)
    }
}

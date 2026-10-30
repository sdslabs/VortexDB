use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use uuid::Uuid;

pub type PointId = Uuid;

/// Type of vector element.
pub type Element = f32;
// pub type ElementHalf = f16; - Unstable https://github.com/rust-lang/rust/issues/116909
pub type ElementByte = u8;

pub type Dimension = usize;

pub type Metadata = serde_json::Map<String, serde_json::Value>;

// Dense Vector and Vector are considered same
// Sparse vector implementation not supported yet. Refer lib/sparse/src/common/sparse_vector.rs
pub type DenseVector = Vec<Element>;

pub type Magic = [u8; 4];

pub enum StoredVector {
    Dense(DenseVector),
}

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
    #[serde(default)]
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PointInput {
    pub id: Option<PointId>,
    pub vector: Option<DenseVector>,
    pub payload: Option<Payload>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Point {
    pub id: PointId,
    pub vector: Option<DenseVector>,
    pub payload: Option<Payload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOp {
    Eq,
    Neq,
    Lt,
    Leq,
    Gt,
    Gteq,
    In,
    Exists,
    NotExists,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldCondition {
    pub key: String,
    pub op: FilterOp,
    #[serde(default)]
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterMode {
    #[default]
    Post,
    Inline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Filter {
    Condition(FieldCondition),
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Not(Box<Filter>),
    Xor(Vec<Filter>),
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
    pub queries: Vec<SearchQueryInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQueryInput {
    pub vector: DenseVector,
    pub similarity: Similarity,
    pub limit: usize,
    #[serde(default)]
    pub ef: Option<usize>, // adding 2 terms in this Struct, filter and method of filteration
    #[serde(default)]
    pub filter: Option<Filter>,
    #[serde(default)]
    pub filter_mode: FilterMode,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SearchResponse {
    pub results: Vec<PointId>,
    pub scores: Vec<f32>,
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

impl Filter {
    pub fn matches(&self, metadata: &Metadata) -> bool {
        match self {
            Filter::Condition(c) => c.matches(metadata),
            Filter::And(fs) => fs.iter().all(|f| f.matches(metadata)),
            Filter::Or(fs) => fs.iter().any(|f| f.matches(metadata)),
            Filter::Not(f) => !f.matches(metadata),
            Filter::Xor(fs) => fs.iter().filter(|f| f.matches(metadata)).count() % 2 == 1,
        }
    }
}

impl FieldCondition {
    fn matches(&self, metadata: &Metadata) -> bool {
        let found = resolve_path(metadata, &self.key);
        match self.op {
            FilterOp::Exists => found.is_some(),
            FilterOp::NotExists => found.is_none(),
            _ => match (found, self.value.as_ref()) {
                (Some(lhs), Some(rhs)) => compare_op(&self.op, lhs, rhs),
                _ => false,
            },
        }
    }
}

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

fn resolve_path<'a>(metadata: &'a Metadata, key: &str) -> Option<&'a serde_json::Value> {
    let mut segments = key.split('.');
    let first = segments.next()?;
    let mut cur = metadata.get(first)?;
    for seg in segments {
        cur = cur.get(seg)?;
    }
    Some(cur)
}

fn value_eq(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    match (a, b) {
        (serde_json::Value::Number(x), serde_json::Value::Number(y)) => {
            matches!((x.as_f64(), y.as_f64()), (Some(x), Some(y)) if x == y)
        }
        _ => a == b,
    }
}

fn ordered_cmp(a: &serde_json::Value, b: &serde_json::Value) -> Option<Ordering> {
    match (a, b) {
        (serde_json::Value::Number(x), serde_json::Value::Number(y)) => {
            x.as_f64()?.partial_cmp(&y.as_f64()?)
        }
        (serde_json::Value::String(x), serde_json::Value::String(y)) => Some(x.cmp(y)),
        (serde_json::Value::Bool(x), serde_json::Value::Bool(y)) => Some(x.cmp(y)),
        _ => None,
    }
}

fn compare_op(op: &FilterOp, a: &serde_json::Value, b: &serde_json::Value) -> bool {
    match op {
        FilterOp::Eq => value_eq(a, b),
        FilterOp::Neq => !value_eq(a, b),
        FilterOp::Lt | FilterOp::Leq | FilterOp::Gt | FilterOp::Gteq => match ordered_cmp(a, b) {
            Some(Ordering::Less) => matches!(op, FilterOp::Lt | FilterOp::Leq),
            Some(Ordering::Equal) => matches!(op, FilterOp::Leq | FilterOp::Gteq),
            Some(Ordering::Greater) => matches!(op, FilterOp::Gt | FilterOp::Gteq),
            None => false,
        },
        FilterOp::In => {
            matches!(b, serde_json::Value::Array(xs) if xs.iter().any(|i| value_eq(i, a)))
        }
        FilterOp::Exists | FilterOp::NotExists => true,
    }
}

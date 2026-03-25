use clap::{Parser, ValueEnum};
use defs::{DenseVector, IndexedVector, PointId, Similarity};
use index::{VectorIndex, error::Result, flat, hnsw, kd_tree};
#[derive(Debug)]
pub enum DatasetType {
    DataSet,
    TestQueries,
    GroundTruth,
}

#[derive(Debug)]
pub struct GroundTruth {
    pub id: PointId,
    pub vector: Vec<PointId>,
}

pub enum BenchIndexer {
    FlatIndex(flat::FlatIndex),
    HnswIndex(hnsw::HnswIndex),
    KdTree(kd_tree::index::KDTree),
}

impl BenchIndexer {
    pub fn search(
        &self,
        vector: DenseVector,
        similarity: Similarity,
        k: usize,
    ) -> Result<Vec<PointId>> {
        match self {
            // Forward the call to the inner FlatIndex
            BenchIndexer::FlatIndex(inner) => inner.search(vector, similarity, k),
            // Forward the call to the inner HnswIndex
            BenchIndexer::HnswIndex(inner) => inner.search(vector, similarity, k),

            BenchIndexer::KdTree(inner) => inner.search(vector, similarity, k),
        }
    }
}

pub struct Dataset {
    pub dimension: usize,
    pub data: Vec<IndexedVector>,
    pub test_queries: Vec<IndexedVector>,
    pub ground_truth: Vec<GroundTruth>,
}

impl Dataset {
    pub fn _new() -> Self {
        Dataset {
            dimension: 128,
            data: Vec::new(),
            test_queries: Vec::new(),
            ground_truth: Vec::new(),
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
#[clap(rename_all = "lower")]
pub enum IndexerType {
    Flat,
    KdTree,
    Hnsw,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Vector Index Benchmark")]
pub struct Args {
    /// The dataset to use for benchmarking
    #[clap(short, long, default_value = "1M")]
    pub dataset: String,

    /// Type of indexer to benchmark
    #[arg(short, long, value_enum, default_value_t = IndexerType::Flat)]
    pub indexer: IndexerType,

    /// Number of nearest neighbors to retrieve
    #[arg(short, long, default_value_t = 10)]
    pub k: usize,

    #[arg(short, long, value_enum, default_value_t = Similarity::Euclidean)]
    pub similarity: Similarity,
}

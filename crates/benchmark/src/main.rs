use std::time::Instant;
pub mod load_dataset;
pub mod test_methods;
mod types;

use crate::load_dataset::{load_dataset_and_test_query, load_ground_truth};
use crate::test_methods::{build_indexer, query_latency, test_accuracy, test_throughput};
use crate::types::DatasetType::{DataSet, GroundTruth, TestQueries};
use crate::types::{Args, BenchIndexer, Dataset, DatasetType, IndexerType};
use clap::Parser;
use index::{flat, hnsw};

fn main() {
    let args: Args = Args::parse();

    // Loading the dataset.
    let mut set: Dataset = Dataset::_new();
    let set_type: DatasetType = DataSet;

    // Data
    println!("Loading dataset...");
    let dataset = args.dataset;
    load_dataset_and_test_query(&mut set, dataset.clone(), set_type);

    //Test query
    println!("Loading test queries...");
    let set_type: DatasetType = TestQueries;
    load_dataset_and_test_query(&mut set, dataset.clone(), set_type);

    //Ground Truth
    println!("Loading ground truth...\n");
    let set_type: DatasetType = GroundTruth;
    load_ground_truth(&mut set, dataset, set_type);

    let a = set.data.len();
    let b = set.test_queries.len();
    let c = set.ground_truth.len();
    println!("Dataset size: {:?}", a);
    println!("Size of test queries: {:?}", b);
    println!(
        "Size of Ground truth: {:?} (Done for sanity check of dataset)\n",
        c
    );

    // Create indexer
    let mut indexer: BenchIndexer;

    match args.indexer {
        IndexerType::Hnsw => {
            let index: hnsw::HnswIndex = hnsw::HnswIndex::new(args.similarity, set.dimension);
            indexer = BenchIndexer::HnswIndex(index);
        }
        IndexerType::Flat => {
            let index = flat::FlatIndex::new();
            indexer = BenchIndexer::FlatIndex(index);
        }

        IndexerType::KdTree => {
            let index: hnsw::HnswIndex = hnsw::HnswIndex::new(args.similarity, set.dimension);
            indexer = BenchIndexer::HnswIndex(index);
        }
    }

    println!("Building dataset ",);
    let start = Instant::now();
    build_indexer(set.data, &mut indexer);
    let duration = start.elapsed();
    println!("Building took {:?} \n", duration);

    println!("Testing accuracy:");
    test_accuracy(
        set.test_queries.clone(),
        set.ground_truth,
        args.similarity,
        &indexer,
        args.k,
    );

    println!("Benchmarking Query Latency");
    query_latency(set.test_queries.clone(), &indexer, args.similarity, args.k);

    println!("Test throughput ");
    test_throughput(set.test_queries, &indexer, args.similarity, args.k);
}

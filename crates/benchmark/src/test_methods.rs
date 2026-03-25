use crate::types::{BenchIndexer, GroundTruth};
use defs::{IndexedVector, Similarity};
use index::VectorIndex;
use index::flat::FlatIndex;
use index::hnsw::HnswIndex;
use index::kd_tree::index::KDTree;
use rayon::prelude::*;
use std::time::{Duration, Instant};

fn build_flat(dataset: Vec<IndexedVector>, indexer: &mut FlatIndex) {
    for i in dataset {
        indexer.insert(i).unwrap();
    }
}
fn build_hnsw(dataset: Vec<IndexedVector>, indexer: &mut HnswIndex) {
    let mut dataset = dataset.into_iter();
    for _i in 0..10000 {
        indexer.insert(dataset.next().unwrap()).unwrap();
    }
}

fn build_kd_tree(dataset: Vec<IndexedVector>, indexer: &mut KDTree) {
    for i in dataset {
        indexer.insert(i).unwrap();
    }
}
pub fn build_indexer(dataset: Vec<IndexedVector>, indexer: &mut BenchIndexer) {
    match indexer {
        BenchIndexer::HnswIndex(indexer) => {
            build_hnsw(dataset, indexer);
        }

        BenchIndexer::FlatIndex(indexer) => {
            build_flat(dataset, indexer);
        }

        BenchIndexer::KdTree(indexer) => {
            build_kd_tree(dataset, indexer);
        }
    }
}

pub fn ttest_accuracy(
    queries: Vec<IndexedVector>,
    ground_truth: Vec<GroundTruth>,
    similarity: Similarity,
    indexer: &BenchIndexer,
    k: usize,
) {
    let length = queries.len();
    let mut results: Vec<GroundTruth> = Vec::with_capacity(length);
    for query in queries.into_iter() {
        let result = indexer.search(query.vector, similarity, k).unwrap();
        let one_unit = GroundTruth {
            id: query.id,
            vector: result,
        };
        results.push(one_unit);
    }

    let mut result_iter = results.into_iter();
    let mut ground_iter = ground_truth.into_iter();
    for _i in 0..length {
        let a = result_iter.next().unwrap();
        let b = ground_iter.next().unwrap();
        let c = &b.vector[..k.min(b.vector.len())];

        let mut count: i32 = 0;
        for j in 0..k {
            if a.vector[j] != b.vector[j] {
                count += 1;
            }
        }

        println!("Point id: ");
        println!("{:?},\n{:?}\n", a.id, b.id);

        println!("Vectors: ");
        println!("{:?},\n{:?}\n", a.vector, c);

        println!("Incorrect matches: {count}");
    }
}
pub fn test_accuracy(
    queries: Vec<IndexedVector>,
    ground_truth: Vec<GroundTruth>,
    similarity: Similarity,
    indexer: &BenchIndexer,
    k: usize,
) {
    let length = ground_truth.len();
    let mut count: i32 = 0;
    queries
        .into_iter()
        .zip(ground_truth)
        .for_each(|(query, truth)| {
            let result_vector = indexer.search(query.vector, similarity, k).unwrap();

            let truth_slice = &truth.vector[..k.min(truth.vector.len())];

            for j in 0..k {
                if j < result_vector.len()
                    && j < truth_slice.len()
                    && result_vector[j] != truth_slice[j]
                {
                    count += 1;
                }
            }
        });
    println!("{:?} wrong out of {:?} \n ", count, length);
}
pub fn query_latency(
    dataset: Vec<IndexedVector>,
    indexer_enum: &BenchIndexer,
    similarity: Similarity,
    k: usize,
) {
    let mut query_time: Vec<Duration> = Vec::with_capacity(dataset.len());

    for query in dataset {
        let start = Instant::now();
        let _ = indexer_enum.search(query.vector, similarity, k);
        let duration = start.elapsed();
        query_time.push(duration);
    }

    query_time.sort_unstable();
    println!("Query time median: {:?}", query_time[query_time.len() / 2]);
    println!(
        "Query time 99 percentiles: {:?}\n",
        query_time[query_time.len() - 1]
    );
}

pub fn test_throughput(
    queries: Vec<IndexedVector>,
    indexer: &BenchIndexer,
    similarity: Similarity,
    k: usize,
) {
    let num_queries = queries.len();
    println!("Starting throughput test with {} queries...", num_queries);

    let start = Instant::now();

    queries.into_par_iter().for_each(|query| {
        let _result = indexer.search(query.vector, similarity, k).unwrap();
    });

    let duration = start.elapsed();
    let total_seconds = duration.as_secs_f64();
    let qps = num_queries as f64 / total_seconds;

    println!("-----------------------------------");
    println!("Throughput Test Results:");
    println!("Total Time: {:.4} s", total_seconds);
    println!("Throughput: {:.2} QPS (Queries Per Second)", qps);
    println!("-----------------------------------");
}

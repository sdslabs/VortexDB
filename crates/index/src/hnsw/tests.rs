use super::*;
use crate::VectorIndex;
use crate::flat::FlatIndex;
use defs::{IndexedVector, Similarity};
use uuid::Uuid;

const ID1: Uuid = Uuid::from_u128(1);
const ID2: Uuid = Uuid::from_u128(2);
const ID3: Uuid = Uuid::from_u128(3);
const ID4: Uuid = Uuid::from_u128(4);
const ID5: Uuid = Uuid::from_u128(5);

#[test]
fn test_entry_point_after_first_insert() {
    let mut index = HnswIndex::new(Similarity::Euclidean, 2);
    let v1 = IndexedVector {
        id: ID1,
        vector: vec![1.0, 0.0],
    };
    assert!(index.insert(v1).is_ok());

    // Entry point should be set to the first inserted id
    assert_eq!(index.index.entry_point, Some(ID1));
    // Layer 0 should contain the point
    assert!(index.index.points_by_layer.first().unwrap().contains(&ID1));
}

#[test]
fn test_connectivity_level0_after_two_inserts() {
    let mut index = HnswIndex::new(Similarity::Euclidean, 2);

    let v1 = IndexedVector {
        id: ID1,
        vector: vec![0.0, 0.0],
    };
    let v2 = IndexedVector {
        id: ID2,
        vector: vec![1.0, 0.0],
    };

    index.insert(v1).unwrap();
    index.insert(v2).unwrap();

    assert!(index.index.nodes.contains_key(&ID1));
    assert!(index.index.nodes.contains_key(&ID2));

    // There should be a base layer (level 0)
    let n1 = index.index.nodes.get(&ID1).unwrap();
    let n2 = index.index.nodes.get(&ID2).unwrap();
    assert!(!n1.neighbors.is_empty());
    assert!(!n2.neighbors.is_empty());

    // At level 0, each should have at least one neighbor; commonly connected to each other
    let nbrs1_lvl0 = &n1.neighbors[0];
    let nbrs2_lvl0 = &n2.neighbors[0];
    assert!(!nbrs1_lvl0.is_empty());
    assert!(!nbrs2_lvl0.is_empty());

    // This check is lenient(probabilistic): only asserts that at least one side linked to the other
    let linked = nbrs1_lvl0.contains(&ID2) || nbrs2_lvl0.contains(&ID1);
    assert!(linked);
}

#[test]
fn test_search_matches_flat_small() {
    let mut flat = FlatIndex::new();
    let mut hnsw = HnswIndex::new(Similarity::Euclidean, 2);

    let data = vec![
        IndexedVector {
            id: ID1,
            vector: vec![1.0, 0.0],
        },
        IndexedVector {
            id: ID2,
            vector: vec![0.0, 1.0],
        },
        IndexedVector {
            id: ID3,
            vector: vec![1.0, 1.0],
        },
        IndexedVector {
            id: ID4,
            vector: vec![0.9, 0.1],
        },
        IndexedVector {
            id: ID5,
            vector: vec![0.2, 0.8],
        },
    ];

    for v in data.clone() {
        flat.insert(v.clone()).unwrap();
        hnsw.insert(v).unwrap();
    }

    let queries = vec![vec![1.0, 0.2], vec![0.1, 0.9]];
    let k = 2;

    for q in queries {
        let flat_ids = flat.search(q.clone(), Similarity::Euclidean, k).unwrap();
        let hnsw_ids = hnsw.search(q.clone(), Similarity::Euclidean, k).unwrap();

        //both return the same number of results and that HNSW matches Flat for this tiny dataset
        assert_eq!(hnsw_ids.len(), k.min(flat_ids.len()));
        assert_eq!(hnsw_ids, flat_ids);
    }
}

#[test]
fn test_search_empty_index_returns_empty() {
    let index = HnswIndex::new(Similarity::Euclidean, 2);
    let res = index
        .search(vec![0.0, 0.0], Similarity::Euclidean, 3)
        .unwrap();
    assert!(res.is_empty());
}

#[test]
fn test_search_k_zero_returns_empty() {
    let mut index = HnswIndex::new(Similarity::Euclidean, 2);
    index
        .insert(IndexedVector {
            id: ID1,
            vector: vec![0.0, 0.0],
        })
        .unwrap();
    let res = index
        .search(vec![0.0, 0.0], Similarity::Euclidean, 0)
        .unwrap();
    assert!(res.is_empty());
}

#[test]
fn test_search_cosine_normalization_basic() {
    let mut index = HnswIndex::new(Similarity::Cosine, 2);
    index
        .insert(IndexedVector {
            id: ID1,
            vector: vec![1.0, 0.0],
        })
        .unwrap();
    index
        .insert(IndexedVector {
            id: ID2,
            vector: vec![0.0, 1.0],
        })
        .unwrap();
    let res = index
        .search(vec![10.0, 0.0], Similarity::Cosine, 1)
        .unwrap();
    assert_eq!(res, vec![ID1]);
}

#[test]
fn test_soft_delete_and_search_skip() {
    let mut index = HnswIndex::new(Similarity::Euclidean, 2);
    index
        .insert(IndexedVector {
            id: ID1,
            vector: vec![0.0, 0.0],
        })
        .unwrap();
    index
        .insert(IndexedVector {
            id: ID2,
            vector: vec![1.0, 0.0],
        })
        .unwrap();
    index
        .insert(IndexedVector {
            id: ID3,
            vector: vec![0.0, 1.0],
        })
        .unwrap();

    let existed = index.delete(ID2).unwrap();
    assert!(existed);
    let n2 = index.index.nodes.get(&ID2).expect("node 2 must exist");
    assert!(n2.deleted);

    // Search near id=2 should not return 2
    let res = index
        .search(vec![0.9, 0.1], Similarity::Euclidean, 2)
        .unwrap();
    assert!(!res.contains(&ID2));

    // Deleting a non-existent id returns false
    let existed = index.delete(ID4).unwrap();
    assert!(!existed);

    // If entry point was 2, it should be updated to a non-deleted id
    if let Some(ep) = index.index.entry_point
        && ep == ID2
    {
        panic!("entry point should have been moved off deleted id");
    }
}

#[test]
fn test_stats_and_deleted_ratio() {
    let mut index = HnswIndex::new(Similarity::Euclidean, 2);
    index
        .insert(IndexedVector {
            id: ID1,
            vector: vec![0.0, 0.0],
        })
        .unwrap();
    index
        .insert(IndexedVector {
            id: ID2,
            vector: vec![1.0, 0.0],
        })
        .unwrap();
    index
        .insert(IndexedVector {
            id: ID3,
            vector: vec![0.0, 1.0],
        })
        .unwrap();
    index
        .insert(IndexedVector {
            id: ID4,
            vector: vec![1.0, 1.0],
        })
        .unwrap();

    index.delete(ID2).unwrap();

    let stats = index.stats();
    assert_eq!(stats.alive + stats.deleted, index.index.nodes.len());
    assert_eq!(stats.deleted, 1);
    assert_eq!(stats.alive, index.index.nodes.len() - 1);

    // Ratio should be > 0 and <= 0.5 for 1/4 deleted
    let ratio = index.deleted_ratio();
    assert!(ratio > 0.0 && ratio <= 0.5, "ratio was {ratio}");

    // Histogram sums to alive count
    let sum_hist: usize = stats.level_histogram.values().sum();
    assert_eq!(sum_hist, stats.alive);
}

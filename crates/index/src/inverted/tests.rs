use super::index::InvertedIndex;
use super::types::PostingList;
use defs::{SparseVector, TokenID};
use uuid::Uuid;

const ID1: Uuid = Uuid::from_u128(1);
const ID2: Uuid = Uuid::from_u128(2);
const ID3: Uuid = Uuid::from_u128(3);

fn sparse(token_ids: &[TokenID], weights: &[f32]) -> SparseVector {
    SparseVector {
        token_ids: token_ids.to_vec(),
        weights: weights.to_vec(),
    }
}

fn sample_index() -> InvertedIndex {
    let mut index = InvertedIndex::new();
    index.insert(ID1, &sparse(&[1, 2], &[1.0, 2.0]));
    index.insert(ID2, &sparse(&[2, 3], &[1.0, 3.0]));
    index.insert(ID3, &sparse(&[4], &[5.0]));
    index
}

#[test]
fn test_posting_list_remove() {
    let mut list = PostingList::default();
    list.insert(ID1, 1.0);
    list.insert(ID2, 2.0);

    assert!(list.remove(ID1));
    assert_eq!(list.iter().collect::<Vec<_>>(), vec![(ID2, 2.0)]);
    assert!(!list.remove(ID1));

    assert!(list.remove(ID2));
    assert!(list.is_empty());
}

#[test]
fn test_search_scores_by_dot_product() {
    let index = sample_index();
    let results = index.search(&sparse(&[2, 3], &[1.0, 1.0]), 10);
    assert_eq!(results, vec![(ID2, 4.0), (ID1, 2.0)]);
}

#[test]
fn test_search_respects_limit() {
    let index = sample_index();
    let results = index.search(&sparse(&[2, 3], &[1.0, 1.0]), 1);
    assert_eq!(results, vec![(ID2, 4.0)]);
}

#[test]
fn test_search_k_zero_returns_empty() {
    let index = sample_index();
    assert!(index.search(&sparse(&[2], &[1.0]), 0).is_empty());
}

#[test]
fn test_search_unknown_token_returns_empty() {
    let index = sample_index();
    assert!(index.search(&sparse(&[99], &[1.0]), 10).is_empty());
}

#[test]
fn test_search_empty_index_returns_empty() {
    let index = InvertedIndex::new();
    assert!(index.search(&sparse(&[1], &[1.0]), 10).is_empty());
}

#[test]
fn test_delete_removes_point_from_results() {
    let mut index = sample_index();

    assert!(index.delete(ID2, &sparse(&[2, 3], &[1.0, 3.0])));

    let results = index.search(&sparse(&[2, 3], &[1.0, 1.0]), 10);
    assert_eq!(results, vec![(ID1, 2.0)]);

    assert!(index.postings.contains_key(&2));
    assert!(!index.postings.contains_key(&3));
}

#[test]
fn test_delete_missing_point_returns_false() {
    let mut index = sample_index();
    let vector = sparse(&[4], &[5.0]);

    assert!(index.delete(ID3, &vector));
    assert!(!index.delete(ID3, &vector));
    assert!(index.search(&vector, 10).is_empty());
}

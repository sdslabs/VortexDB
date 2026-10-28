use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, hash_map::Entry},
};

use defs::{OrdF32, PointId, SparseVector, TokenID};

use crate::inverted::types::PostingList;

pub struct InvertedIndex {
    pub(super) postings: HashMap<TokenID, PostingList>,
}

impl Default for InvertedIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl InvertedIndex {
    pub fn new() -> Self {
        InvertedIndex {
            postings: HashMap::new(),
        }
    }

    pub fn insert(&mut self, point_id: PointId, vector: &SparseVector) {
        for (&token, &weight) in vector.token_ids.iter().zip(&vector.weights) {
            self.postings
                .entry(token)
                .or_default()
                .insert(point_id, weight);
        }
    }

    pub fn search(&self, query: &SparseVector, k: usize) -> Vec<(PointId, f32)> {
        if k == 0 {
            return Vec::new();
        }
        let mut scores: HashMap<PointId, f32> = HashMap::new();
        for (&token, &query_weight) in query.token_ids.iter().zip(&query.weights) {
            let Some(list) = self.postings.get(&token) else {
                continue;
            };
            for (point_id, weight) in list.iter() {
                *scores.entry(point_id).or_default() += query_weight * weight;
            }
        }

        let mut heap: BinaryHeap<(Reverse<OrdF32>, PointId)> =
            BinaryHeap::with_capacity(k.min(scores.len()) + 1);
        for (point_id, score) in scores {
            heap.push((Reverse(OrdF32(score)), point_id));
            if heap.len() > k {
                heap.pop();
            }
        }

        heap.into_sorted_vec()
            .into_iter()
            .map(|(Reverse(score), point_id)| (point_id, score.0))
            .collect()
    }

    pub fn delete(&mut self, point_id: PointId, vector: &SparseVector) -> bool {
        let mut removed = false;
        for &token in &vector.token_ids {
            if let Entry::Occupied(mut entry) = self.postings.entry(token) {
                removed |= entry.get_mut().remove(point_id);
                if entry.get().is_empty() {
                    entry.remove();
                }
            }
        }
        removed
    }
}

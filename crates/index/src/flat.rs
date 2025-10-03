use defs::{DbError, DenseVector, DistanceOrderedVector, IndexedVector, PointId, Similarity};

use crate::{VectorIndex, distance};

pub struct FlatIndex {
    index: Vec<IndexedVector>,
}

impl FlatIndex {
    pub fn new() -> Self {
        Self { index: Vec::new() }
    }

    pub fn build(vectors: Vec<IndexedVector>) -> Self {
        FlatIndex { index: vectors }
    }
}

impl Default for FlatIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorIndex for FlatIndex {
    fn insert(&mut self, vector: IndexedVector) -> Result<(), DbError> {
        self.index.push(vector);
        Ok(())
    }

    fn delete(&mut self, point_id: PointId) -> Result<bool, DbError> {
        if let Some(pos) = self.index.iter().position(|vector| vector.id == point_id) {
            self.index.remove(pos);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn search(
        &self,
        query_vector: DenseVector,
        similarity: Similarity,
        k: usize,
    ) -> Result<Vec<PointId>, DbError> {
        let scores = self
            .index
            .iter()
            .map(|point| DistanceOrderedVector {
                distance: distance(&point.vector, &query_vector, similarity),
                query_vector: &query_vector,
                point_id: Some(point.id),
            })
            .collect::<Vec<_>>();

        // select k smallest elements in scores using a max heap
        let mut heap = std::collections::BinaryHeap::<DistanceOrderedVector>::new();
        for score in scores {
            if heap.len() < k {
                heap.push(score);
            } else if score < *heap.peek().unwrap() {
                heap.pop();
                heap.push(score);
            }
        }
        Ok(heap
            .into_sorted_vec()
            .into_iter()
            .map(|v| v.point_id.unwrap())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_flat_index_new() {
        let index = FlatIndex::new();
        assert_eq!(index.index.len(), 0);
    }

    #[test]
    fn test_flat_index_build() {
        let vectors = vec![
            IndexedVector {
                id: Uuid::new_v4(),
                vector: vec![1.0, 2.0, 3.0],
            },
            IndexedVector {
                id: Uuid::new_v4(),
                vector: vec![4.0, 5.0, 6.0],
            },
        ];
        let index = FlatIndex::build(vectors.clone());
        assert_eq!(index.index, vectors);
    }

    #[test]
    fn test_insert() {
        let mut index = FlatIndex::new();
        let vector = IndexedVector {
            id: Uuid::new_v4(),
            vector: vec![1.0, 2.0, 3.0],
        };

        assert!(index.insert(vector.clone()).is_ok());
        assert_eq!(index.index.len(), 1);
        assert_eq!(index.index[0], vector);
    }

    #[test]
    fn test_delete_existing() {
        let mut index = FlatIndex::new();
        let existing_id = Uuid::new_v4();
        let vector = IndexedVector {
            id: existing_id,
            vector: vec![1.0, 2.0, 3.0],
        };
        index.insert(vector).unwrap();

        let result = index.delete(existing_id).unwrap();
        assert!(result);
        assert_eq!(index.index.len(), 0);
    }

    #[test]
    fn test_delete_non_existing() {
        let mut index = FlatIndex::new();
        let vector = IndexedVector {
            id: Uuid::new_v4(),
            vector: vec![1.0, 2.0, 3.0],
        };
        index.insert(vector).unwrap();

        let result = index.delete(Uuid::new_v4()).unwrap();
        assert!(!result);
        assert_eq!(index.index.len(), 1);
    }

    #[test]
    fn test_search_euclidean() {
        let mut index = FlatIndex::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        index
            .insert(IndexedVector {
                id: id1,
                vector: vec![1.0, 1.0],
            })
            .unwrap();
        index
            .insert(IndexedVector {
                id: id2,
                vector: vec![2.0, 2.0],
            })
            .unwrap();
        index
            .insert(IndexedVector {
                id: id3,
                vector: vec![10.0, 10.0],
            })
            .unwrap();

        let results = index
            .search(vec![0.0, 0.0], Similarity::Euclidean, 2)
            .unwrap();
        assert_eq!(results, vec![id1, id2]);
    }

    #[test]
    fn test_search_cosine() {
        let mut index = FlatIndex::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        index
            .insert(IndexedVector {
                id: id1,
                vector: vec![1.0, 0.0],
            })
            .unwrap();
        index
            .insert(IndexedVector {
                id: id2,
                vector: vec![0.5, 0.5],
            })
            .unwrap();
        index
            .insert(IndexedVector {
                id: id3,
                vector: vec![0.0, 1.0],
            })
            .unwrap();

        let results = index.search(vec![1.0, 1.0], Similarity::Cosine, 2).unwrap();
        assert_eq!(results, vec![id2, id1]);
    }

    #[test]
    fn test_search_manhattan() {
        let mut index = FlatIndex::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        index
            .insert(IndexedVector {
                id: id1,
                vector: vec![1.0, 1.0],
            })
            .unwrap();
        index
            .insert(IndexedVector {
                id: id2,
                vector: vec![2.0, 2.0],
            })
            .unwrap();
        index
            .insert(IndexedVector {
                id: id3,
                vector: vec![5.0, 5.0],
            })
            .unwrap();

        let results = index
            .search(vec![0.0, 0.0], Similarity::Manhattan, 2)
            .unwrap();
        assert_eq!(results, vec![id1, id2]);
    }

    #[test]
    fn test_search_hamming() {
        let mut index = FlatIndex::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        index
            .insert(IndexedVector {
                id: id1,
                vector: vec![1.0, 0.0, 1.0, 1.0],
            })
            .unwrap();
        index
            .insert(IndexedVector {
                id: id2,
                vector: vec![1.0, 0.0, 0.0, 0.0],
            })
            .unwrap();
        index
            .insert(IndexedVector {
                id: id3,
                vector: vec![0.0, 0.0, 0.0, 0.0],
            })
            .unwrap();

        let results = index
            .search(vec![1.0, 0.0, 0.0, 0.0], Similarity::Hamming, 2)
            .unwrap();
        assert_eq!(results, vec![id2, id3]);
    }

    #[test]
    fn test_default() {
        let index = FlatIndex::default();
        assert_eq!(index.index.len(), 0);
    }
}

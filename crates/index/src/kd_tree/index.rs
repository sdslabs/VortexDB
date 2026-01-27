// use super::helpers::{collect_active_vectors, is_unbalanced, should_rebuild_global};
use super::types::{KDTreeNode, Neighbor};
use crate::{VectorIndex, distance};
use defs::{DbError, DenseVector, IndexedVector, OrdF32, PointId, Similarity};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
    vec,
};
use uuid::Uuid;

pub struct KDTree {
    pub dim: usize,
    pub root: Option<Box<KDTreeNode>>,
    // In memory point ids, to check existence before O(n) deletion logic
    pub point_ids: HashSet<PointId>,
    // Rebuild tracking
    pub total_nodes: usize,
    pub deleted_count: usize,
}

impl KDTree {
    // Build an empty index with no points
    pub fn build_empty(dim: usize) -> Self {
        KDTree {
            dim,
            root: None,
            point_ids: HashSet::new(),
            total_nodes: 0,
            deleted_count: 0,
        }
    }

    // Builds the vector index from provided vectors, there should atleast be single vector for dim calculation
    pub fn build(mut vectors: Vec<IndexedVector>) -> Result<Self, DbError> {
        if vectors.is_empty() {
            Err(DbError::IndexInitError)
        } else {
            let dim = vectors[0].vector.len();

            let mut point_ids = HashSet::with_capacity(vectors.len());
            for indexed_vector in vectors.iter() {
                point_ids.insert(indexed_vector.id);
            }

            let root_node = Self::build_recursive(&mut vectors, 0, dim);
            Ok(KDTree {
                dim,
                root: Some(root_node),
                point_ids,
                total_nodes: vectors.len(),
                deleted_count: 0,
            })
        }
    }

    // Builds the tree recursively with given vectors and returns the pointer of the root node
    pub fn build_recursive(
        vectors: &mut [IndexedVector],
        depth: usize,
        dim: usize,
    ) -> Box<KDTreeNode> {
        if vectors.is_empty() {
            panic!("Cannot build from an empty slice recursively");
        }

        let axis = depth % dim;
        let mid_idx = vectors.len() / 2;

        vectors.select_nth_unstable_by(mid_idx, |a, b| {
            let a_at_axis = a.vector[axis];
            let b_at_axis = b.vector[axis];
            a_at_axis.partial_cmp(&b_at_axis).unwrap_or(Ordering::Equal)
        });

        // Using swap so that we don't need to clone the whole vector
        let mut median_vec = IndexedVector {
            id: Uuid::new_v4(),
            vector: vec![],
        }; // dummy
        std::mem::swap(&mut vectors[mid_idx], &mut median_vec);

        let (left_points, right_points_with_median) = vectors.split_at_mut(mid_idx);
        let right_points = &mut right_points_with_median[1..]; // Exclude the swapped-out median

        let left = if left_points.is_empty() {
            None
        } else {
            Some(Self::build_recursive(left_points, depth + 1, dim))
        };

        let right = if right_points.is_empty() {
            None
        } else {
            Some(Self::build_recursive(right_points, depth + 1, dim))
        };

        let left_size = left.as_ref().map_or(0, |n| n.subtree_size);
        let right_size = right.as_ref().map_or(0, |n| n.subtree_size);

        Box::new(KDTreeNode {
            indexed_vector: median_vec,
            left,
            right,
            is_deleted: false,
            axis,
            subtree_size: left_size + right_size + 1,
        })
    }

    pub fn insert_point(&mut self, new_vector: IndexedVector) {
        // Add to point_ids
        self.point_ids.insert(new_vector.id);
        self.total_nodes += 1;

        // use a traverse function to get the final leaf where this belongs
        if self.root.is_none() {
            self.root = Some(Box::new(KDTreeNode {
                indexed_vector: new_vector,
                left: None,
                right: None,
                is_deleted: false,
                axis: 0,
                subtree_size: 1,
            }));
            return;
        }

        let mut path: Vec<(usize, bool)> = Vec::new();
        let dim = self.dim;

        let mut current_link = &mut self.root;
        let mut depth = 0;

        while let Some(node_box) = current_link {
            let current_node = node_box.as_mut();
            let axis = current_node.axis;

            current_node.subtree_size += 1;

            let va = new_vector.vector[axis];
            let vb = current_node.indexed_vector.vector[axis];

            let go_left = va <= vb;
            path.push((depth, go_left));

            if go_left {
                current_link = &mut current_node.left;
            } else {
                current_link = &mut current_node.right;
            }
            depth += 1;
        }

        // Assign the new node to current link which is &mut Option<Box<KDTreeNode>>
        let new_node = Box::new(KDTreeNode {
            indexed_vector: new_vector,
            left: None,
            right: None,
            is_deleted: false,
            axis: depth % dim,
            subtree_size: 1,
        });

        *current_link = Some(new_node);

        self.check_and_rebalance(&path);
    }

    fn rebuild_at_depth(&mut self, path: &[(usize, bool)], target_depth: usize) {
        let dim = self.dim;

        // Navigate to parent of target node
        if target_depth == 0 {
            // Rebuild root
            if let Some(root) = self.root.take() {
                let old_size = root.subtree_size;
                let mut vectors = Self::collect_active_vectors(*root);
                let new_size = vectors.len();
                if !vectors.is_empty() {
                    self.root = Some(Self::build_recursive(&mut vectors, 0, dim));
                }
                // Update global counts as deleted nodes were purged
                self.total_nodes -= old_size - new_size;
                self.deleted_count = 0;
            }
        } else {
            // Navigate to target node
            let mut current_link = &mut self.root;
            for (_depth, go_left) in path.iter().take(target_depth) {
                let node = current_link.as_mut().unwrap();
                current_link = if *go_left {
                    &mut node.left
                } else {
                    &mut node.right
                };
            }

            // Rebuild tree at current link
            if let Some(subtree_root) = current_link.take() {
                let old_size = subtree_root.subtree_size;
                let mut vectors = Self::collect_active_vectors(*subtree_root);
                let new_size = vectors.len();

                if !vectors.is_empty() {
                    *current_link = Some(Self::build_recursive(&mut vectors, target_depth, dim));
                }

                // Only update ancestors if size changed (deleted nodes were purged)
                if old_size != new_size {
                    let size_diff = old_size - new_size;
                    self.subtract_size_from_ancestors(path, target_depth, size_diff);

                    self.total_nodes -= size_diff;
                    self.deleted_count = self.deleted_count.saturating_sub(size_diff);
                }
            }
        }
    }

    fn subtract_size_from_ancestors(
        &mut self,
        path: &[(usize, bool)],
        up_to_depth: usize,
        diff: usize,
    ) {
        let mut current = &mut self.root;
        for (_, go_left) in path.iter().take(up_to_depth) {
            if let Some(node) = current {
                node.subtree_size -= diff;
                current = if *go_left {
                    &mut node.left
                } else {
                    &mut node.right
                };
            }
        }
    }

    fn check_and_rebalance(&mut self, path: &[(usize, bool)]) {
        // Find the shallowest (closest to root) depth where imbalance occurs
        // so that rebuilding fixes the largest unbalanced subtree
        let mut unbalanced_depth: Option<usize> = None;

        let mut current = self.root.as_ref();

        // Check root first (depth 0)
        if let Some(node) = current
            && Self::is_unbalanced(node)
        {
            unbalanced_depth = Some(0);
        }

        // Then traverse the path and check each node
        // Once we find the shallowest unbalanced node, break immediately
        for (idx, (_depth, go_left)) in path.iter().enumerate() {
            if unbalanced_depth.is_some() {
                break;
            }

            if let Some(node) = current {
                current = if *go_left {
                    node.left.as_ref()
                } else {
                    node.right.as_ref()
                };

                // Check the child node we just moved to (at depth idx + 1)
                if let Some(child) = current
                    && Self::is_unbalanced(child)
                {
                    unbalanced_depth = Some(idx + 1);
                    break;
                }
            }
        }

        if let Some(target_depth) = unbalanced_depth {
            self.rebuild_at_depth(path, target_depth);
        }
    }

    // Returns true if point found and deleted, else false
    pub fn delete_point(&mut self, point_id: &PointId) -> bool {
        if self.point_ids.contains(point_id) {
            let deleted = Self::find_and_mark_deleted(&mut self.root, *point_id);
            if deleted {
                self.deleted_count += 1;
                self.point_ids.remove(point_id);
            }

            if Self::should_rebuild_global(self.total_nodes, self.deleted_count)
                && let Some(root) = self.root.take()
            {
                let mut vectors = Self::collect_active_vectors(*root);
                if !vectors.is_empty() {
                    self.root = Some(Self::build_recursive(&mut vectors, 0, self.dim));
                }

                self.total_nodes = vectors.len();
                self.deleted_count = 0;
            }

            return deleted;
        }
        false
    }

    fn find_and_mark_deleted(node_opt: &mut Option<Box<KDTreeNode>>, target_id: PointId) -> bool {
        if let Some(node) = node_opt {
            if node.indexed_vector.id == target_id {
                node.is_deleted = true;
                return true;
            }

            // Search left first then right
            Self::find_and_mark_deleted(&mut node.left, target_id)
                || Self::find_and_mark_deleted(&mut node.right, target_id)
        } else {
            false
        }
    }

    pub fn search_top_k(
        &self,
        query_vector: DenseVector,
        k: usize,
        dist_type: Similarity,
    ) -> Vec<(PointId, f32)> {
        //Searches for top k closest vectors according to specified metric

        if self.root.is_none() || k == 0 {
            return Vec::new();
        }

        let mut best_neighbours = BinaryHeap::with_capacity(k);

        self.search_recursive(
            &self.root,
            &query_vector,
            k,
            &mut best_neighbours,
            dist_type,
        );

        best_neighbours
            .into_sorted_vec()
            .iter()
            .map(|neighbor| (neighbor.id, neighbor.distance.into_inner()))
            .collect()
    }

    fn search_recursive(
        &self,
        node_opt: &Option<Box<KDTreeNode>>,
        query_vector: &DenseVector,
        k: usize,
        heap: &mut BinaryHeap<Neighbor>,
        dist_type: Similarity,
    ) {
        // Base case is that we hit a leaf node don't do anything
        if let Some(node) = node_opt {
            let axis = node.axis;

            let (near_side, far_side) = if query_vector[axis] <= node.indexed_vector.vector[axis] {
                (&node.left, &node.right)
            } else {
                (&node.right, &node.left)
            };

            // Recurse on near side first
            self.search_recursive(near_side, query_vector, k, heap, dist_type);

            if !node.is_deleted {
                // TODO: Possible overhead, here heap stores sqrt euclidean distance, we can eliminate that by storing squared distances in case of euclidean
                let distance = distance(query_vector, &node.indexed_vector.vector, dist_type);
                if heap.len() < k {
                    heap.push(Neighbor {
                        distance: OrdF32::new(distance),
                        id: node.indexed_vector.id,
                    });
                } else if distance < heap.peek().unwrap().distance.into_inner() {
                    heap.pop();
                    heap.push(Neighbor {
                        distance: OrdF32::new(distance),
                        id: node.indexed_vector.id,
                    });
                }
            }

            // Pruning on the farther side to check if there are better candidates
            // Use <= to handle ties: when axis_diff == current worst distance, there could be
            // a point on the far side with the same distance that should be included
            let axis_diff = (query_vector[axis] - node.indexed_vector.vector[axis]).abs();
            let should_search_far = match dist_type {
                Similarity::Euclidean | Similarity::Manhattan => {
                    heap.len() < k || axis_diff <= heap.peek().unwrap().distance.into_inner()
                }
                _ => true, // Cosine/Hamming - no effective pruning, always search
            };

            if should_search_far {
                self.search_recursive(far_side, query_vector, k, heap, dist_type);
            }
        }
    }
}

impl VectorIndex for KDTree {
    fn insert(&mut self, vector: IndexedVector) -> Result<(), DbError> {
        self.insert_point(vector);
        Ok(())
    }

    fn delete(&mut self, point_id: PointId) -> Result<bool, DbError> {
        Ok(self.delete_point(&point_id))
    }

    fn search(
        &self,
        query_vector: DenseVector,
        similarity: Similarity,
        k: usize,
    ) -> Result<Vec<PointId>, DbError> {
        if matches!(similarity, Similarity::Cosine | Similarity::Hamming) {
            return Err(DbError::UnsupportedSimilarity);
        }

        let results = self.search_top_k(query_vector, k, similarity);
        Ok(results.into_iter().map(|(id, _)| id).collect())
    }
}

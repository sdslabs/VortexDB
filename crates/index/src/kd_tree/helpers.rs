use crate::kd_tree::index::KDTree;

use super::types::KDTreeNode;
use defs::IndexedVector;

impl KDTree {
    pub const BALANCE_THRESHOLD: f32 = 0.7;
    pub const DELETE_REBUILD_RATIO: f32 = 0.25;

    /// Checks if a node is unbalanced based on the balance threshold
    pub fn is_unbalanced(node: &KDTreeNode) -> bool {
        let left_size = node.left.as_ref().map_or(0, |n| n.subtree_size);
        let right_size = node.right.as_ref().map_or(0, |n| n.subtree_size);
        let max_child = left_size.max(right_size);

        max_child as f32 > Self::BALANCE_THRESHOLD * node.subtree_size as f32
    }

    /// Recursively collects non-deleted vectors from the tree
    pub fn collect_recursive(node: KDTreeNode, result: &mut Vec<IndexedVector>) {
        if !node.is_deleted {
            result.push(node.indexed_vector);
        }
        if let Some(left) = node.left {
            Self::collect_recursive(*left, result);
        }
        if let Some(right) = node.right {
            Self::collect_recursive(*right, result);
        }
    }

    /// Collects all active (non-deleted) vectors from a subtree
    pub fn collect_active_vectors(node: KDTreeNode) -> Vec<IndexedVector> {
        let mut result = Vec::with_capacity(node.subtree_size);
        Self::collect_recursive(node, &mut result);
        result
    }

    /// Checks if the tree should be globally rebuilt based on deletion ratio
    pub fn should_rebuild_global(total_nodes: usize, deleted_count: usize) -> bool {
        total_nodes > 0 && (deleted_count as f32 / total_nodes as f32) > Self::DELETE_REBUILD_RATIO
    }
}

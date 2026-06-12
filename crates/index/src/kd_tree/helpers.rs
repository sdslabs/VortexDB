use crate::kd_tree::index::KDTree;

use super::types::KDTreeNode;
use defs::IndexedVector;

impl KDTree {
    /// Checks if a node is unbalanced based on the balance threshold
    pub fn is_unbalanced(&self, node: &KDTreeNode) -> bool {
        let left_size = node.left.as_ref().map_or(0, |n| n.subtree_size);
        let right_size = node.right.as_ref().map_or(0, |n| n.subtree_size);
        let max_child = left_size.max(right_size);

        max_child as f32 > self.config.balance_threshold * node.subtree_size as f32
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
    pub fn should_rebuild_global(&self) -> bool {
        self.total_nodes > 0
            && (self.deleted_count as f32 / self.total_nodes as f32)
                > self.config.delete_rebuild_ratio
    }
}

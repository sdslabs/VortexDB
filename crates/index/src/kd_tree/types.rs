use defs::{IndexedVector, OrdF32, PointId};

// the node which will be the part of the KD Tree
pub struct KDTreeNode {
    pub indexed_vector: IndexedVector,
    pub left: Option<Box<KDTreeNode>>,
    pub right: Option<Box<KDTreeNode>>,
    pub is_deleted: bool,
    pub axis: usize,
    pub subtree_size: usize,
}

// The struct definition which is present in max heap while search
// distance is first for correct Ord derivation (primary sort key)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Neighbor {
    pub distance: OrdF32,
    pub id: PointId,
}

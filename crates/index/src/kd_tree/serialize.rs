use std::collections::HashSet;
use std::io::{Cursor, Read, Write};

use super::KD_TREE_MAGIC_BYTES;
use super::constants::{DELETED_MASK, NODE_MARKER_BYTE, SKIP_MARKER_BYTE};
use super::index::KDTree;
use super::types::KDTreeNode;
use crate::{IndexSnapshot, IndexType, SerializableIndex};
use bincode;
use defs::{DbError, IndexedVector, PointId};
use serde::{Deserialize, Serialize};
use storage::StorageEngine;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct KDTreeMetadata {
    pub dim: usize,
    pub total_nodes: usize,
    pub deleted_count: usize,
}

impl SerializableIndex for KDTree {
    fn serialize_topology(&self) -> Result<Vec<u8>, DbError> {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        serialize_topology_recursive(&self.root, &mut cursor)?;
        Ok(buffer)
    }

    fn serialize_metadata(&self) -> Result<Vec<u8>, DbError> {
        let mut buffer = Vec::new();
        let km = KDTreeMetadata {
            dim: self.dim,
            total_nodes: self.total_nodes,
            deleted_count: self.deleted_count,
        };
        let metadata_bytes = bincode::serialize(&km).map_err(|e| {
            DbError::SerializationError(format!("Failed to serailize KD Tree Metadata: {}", e))
        })?;
        buffer.extend_from_slice(metadata_bytes.as_slice());
        Ok(buffer)
    }

    fn snapshot(&self) -> Result<IndexSnapshot, DbError> {
        let topology_bytes = self.serialize_topology()?;
        let metadata_bytes = self.serialize_metadata()?;
        Ok(IndexSnapshot {
            index_type: crate::IndexType::KDTree,
            magic: KD_TREE_MAGIC_BYTES,
            topology_b: topology_bytes,
            metadata_b: metadata_bytes,
        })
    }

    fn populate_vectors(&mut self, storage: &dyn StorageEngine) -> Result<(), DbError> {
        populate_vectors_recursive(&mut self.root, storage)?;
        Ok(())
    }
}

impl KDTree {
    pub fn deserialize(
        IndexSnapshot {
            index_type,
            magic,
            topology_b,
            metadata_b,
        }: &IndexSnapshot,
    ) -> Result<KDTree, DbError> {
        if index_type != &IndexType::KDTree {
            return Err(DbError::SerializationError(
                "Invalid index type".to_string(),
            ));
        }

        if magic != &KD_TREE_MAGIC_BYTES {
            return Err(DbError::SerializationError(
                "Invalid magic bytes".to_string(),
            ));
        }

        let metadata: KDTreeMetadata =
            bincode::deserialize(metadata_b.as_slice()).map_err(|e| {
                DbError::SerializationError(format!(
                    "Failed to deserailize KD Tree Metadata: {}",
                    e
                ))
            })?;

        let mut buf = Cursor::new(topology_b);
        let mut non_deleted = HashSet::new();
        let root = deserialize_topology_recursive(metadata.dim, 0, &mut buf, &mut non_deleted)?;

        Ok(KDTree {
            dim: metadata.dim,
            root,
            point_ids: non_deleted,
            total_nodes: metadata.total_nodes,
            deleted_count: metadata.deleted_count,
            config: Default::default(),
        })
    }
}

// helper functions

fn serialize_topology_recursive(
    current_opt: &Option<Box<KDTreeNode>>,
    buffer: &mut Cursor<&mut Vec<u8>>,
) -> Result<(), DbError> {
    if let Some(current) = current_opt {
        let mut marker = NODE_MARKER_BYTE;
        if current.is_deleted {
            marker |= DELETED_MASK;
        }
        buffer
            .write_all(&[marker])
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        let uuid_bytes = current.indexed_vector.id.to_bytes_le();
        buffer
            .write_all(&uuid_bytes)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        // serialize left subtree topology
        serialize_topology_recursive(&current.left, buffer)?;
        // serialize right subtree topology
        serialize_topology_recursive(&current.right, buffer)?;
    } else {
        buffer
            .write_all(&[SKIP_MARKER_BYTE])
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
    }
    Ok(())
}

fn populate_vectors_recursive(
    node: &mut Option<Box<KDTreeNode>>,
    storage: &dyn StorageEngine,
) -> Result<(), DbError> {
    if let Some(node) = node {
        let vector = storage
            .get_vector(node.indexed_vector.id)
            .map_err(|e| {
                DbError::SerializationError(format!("Could not get vector from storage: {e}"))
            })?
            .ok_or_else(|| {
                DbError::SerializationError(format!(
                    "Failed to locate vector for id: {}",
                    node.indexed_vector.id
                ))
            })?;
        node.indexed_vector.vector = vector;

        populate_vectors_recursive(&mut node.left, storage)?;
        populate_vectors_recursive(&mut node.right, storage)?;
    }
    Ok(())
}

fn deserialize_topology_recursive(
    dimensions: usize,
    depth: usize,
    buffer: &mut Cursor<&Vec<u8>>,
    non_deleted: &mut HashSet<PointId>,
) -> Result<Option<Box<KDTreeNode>>, DbError> {
    let mut current_marker: [u8; 1] = [0u8; 1];
    buffer.read_exact(&mut current_marker).map_err(|e| {
        DbError::SerializationError(format!("Failed to deserialize KD Topology: {}", e))
    })?;

    if current_marker[0] == SKIP_MARKER_BYTE {
        return Ok(None);
    }

    let mut uuid_bytes = [0u8; 16];
    buffer.read_exact(&mut uuid_bytes).map_err(|e| {
        DbError::SerializationError(format!("Failed to deserialize KD Topology: {}", e))
    })?;
    let uuid = Uuid::from_bytes_le(uuid_bytes);
    let indexed_vector = IndexedVector {
        id: uuid,
        vector: Vec::new(),
    };

    let is_deleted = current_marker[0] & DELETED_MASK == DELETED_MASK;
    if !is_deleted {
        non_deleted.insert(uuid);
    }

    // pre order deserialization
    let lower_dim = (depth + 1) % dimensions;
    let left_node = deserialize_topology_recursive(dimensions, lower_dim, buffer, non_deleted)?;
    let right_node = deserialize_topology_recursive(dimensions, lower_dim, buffer, non_deleted)?;

    let left_size = left_node.as_ref().map_or(0, |n| n.subtree_size);
    let right_size = right_node.as_ref().map_or(0, |n| n.subtree_size);

    let current_node = KDTreeNode {
        indexed_vector,
        left: left_node,
        right: right_node,
        is_deleted,
        axis: depth,
        subtree_size: left_size + right_size + 1,
    };

    Ok(Some(Box::new(current_node)))
}

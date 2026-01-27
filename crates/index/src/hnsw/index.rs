use std::collections::HashMap;

use defs::{DbError, DenseVector, Dimension, IndexedVector, PointId, Similarity};
use uuid::Uuid;

use crate::VectorIndex;

use super::types::{HnswStats, LevelGenerator, Node, PointIndexation};
use std::cmp::{max, min};

pub struct HnswIndex {
    // Construction/search parameters
    pub ef_construction: usize,
    // Layered point storage and entry point
    pub index: PointIndexation,
    // Cached dimension of stored vectors
    pub data_dimension: Dimension,
    // Default query beam width (ef); recommended ef ≥ k at query time
    pub ef: usize,
    // In-memory vector cache owned by the index
    cache: HashMap<PointId, DenseVector>,
    // Fixed metric for this index; used consistently in insert and search
    pub similarity: Similarity,
}

impl HnswIndex {
    pub fn new(similarity: Similarity, data_dimension: Dimension) -> Self {
        let max_connections = 16;
        let max_connections_0 = 32; // M0 = 2 * M (common default)
        let max_layer = 16;
        let ef_construction = 200;
        let ef = 100;

        let level_generator = LevelGenerator::from_m(max_connections);
        let index = PointIndexation {
            max_connections,
            max_connections_0,
            max_layer,
            points_by_layer: vec![Vec::new(); max_layer],
            nodes: HashMap::new(),
            entry_point: None,
            level_generator,
        };

        Self {
            ef_construction,
            index,
            data_dimension,
            ef,
            cache: HashMap::new(),
            similarity,
        }
    }

    /// Returns a slice of the stored vector for the given PointId.
    /// TODO: integrate this cache with an in-memory store backed by RocksDB; on cache miss,
    /// fetch from storage, populate the cache, and return a stable slice.
    pub(super) fn get_vec(&self, id: PointId) -> Result<&Vec<f32>, DbError> {
        self.cache.get(&id).ok_or(DbError::PointNotFound { id })
    }
}

impl VectorIndex for HnswIndex {
    /// Insert a new point
    /// - sample a random level for the new node
    /// - if empty, set entry point to the new id and return
    /// - greedy descend from current entry to l+1 to get a pivot
    /// - for each level down to 0: ef-construction, diversity pruning, bidirectional connect with caps
    /// - if l is above current max level, update the entry point
    fn insert(&mut self, vector: IndexedVector) -> Result<(), DbError> {
        if self.index.nodes.contains_key(&vector.id) {
            return Err(DbError::PointAlreadyExists { id: vector.id });
        }

        let dim = vector.vector.len();
        if dim != self.data_dimension {
            return Err(DbError::InvalidDimension {
                expected: self.data_dimension,
                got: dim,
            });
        }

        let new_id: PointId = vector.id;

        let mut query_vec = vector.vector.clone();
        self.normalize_if_cosine(&mut query_vec);

        self.cache.insert(new_id, query_vec.clone());

        let mut rng = rand::rng();
        let l: u8 = self
            .index
            .level_generator
            .sample_level(&mut rng, self.index.max_layer);

        let node = Node {
            id: new_id,
            level: l,
            neighbors: vec![vec![]; (l as usize) + 1],
            deleted: false,
        };

        let needed_layers = (l as usize) + 1;
        if self.index.points_by_layer.len() < needed_layers {
            self.index.points_by_layer.resize(needed_layers, Vec::new());
        }
        self.index.nodes.insert(new_id, node);
        for layer in 0..=l as usize {
            self.index.points_by_layer[layer].push(new_id);
        }

        if self.index.entry_point.is_none() {
            self.index.entry_point = Some(new_id);
            return Ok(());
        }

        let mut ep = self.index.entry_point.unwrap();
        let current_max_level = self
            .index
            .nodes
            .get(&ep)
            .map(|n| n.level as usize)
            .unwrap_or(0);
        if current_max_level > (l as usize) {
            for level in ((l as usize + 1)..=current_max_level).rev() {
                ep = self.greedy_search_layer(ep, level, &query_vec)?;
            }
        }

        for level in (0..=min(l as usize, current_max_level)).rev() {
            let w = self.search_layer_for_insert(ep, level, &query_vec, self.ef_construction)?;

            let m_level = if level == 0 {
                self.index.max_connections_0
            } else {
                self.index.max_connections
            };
            let chosen = self.select_neighbors_heuristic(&w, m_level)?;
            self.connect_bidirectional(new_id, &chosen, level, m_level)?;
            if let Some((closest_id, _)) = w.first() {
                ep = *closest_id;
            }
        }
        if (l as usize) > current_max_level {
            self.index.entry_point = Some(new_id);
        }

        Ok(())
    }

    /// Delete a point (soft)
    /// - mark node as deleted and clear its cached vector
    /// - traversals skip deleted nodes
    /// - if entry point was deleted, move it to the highest-level non-deleted node (or None)
    fn delete(&mut self, point_id: PointId) -> Result<bool, DbError> {
        if let Some(node) = self.index.nodes.get_mut(&point_id) {
            if node.deleted {
                return Ok(false);
            }
            node.deleted = true;
            self.cache.remove(&point_id);
            if self.index.entry_point == Some(point_id) {
                self.index.entry_point = self.pick_entry();
            }
            return Ok(true);
        }
        Ok(false)
    }
    /// Search for top-k ids
    /// - normalize query for cosine (1 − cos)
    /// - pick a non-deleted entry point
    /// - greedy descend from the top layer to level 1
    /// - run ef-best-first at level 0 with ef0 = max(ef, k)
    /// - return up to k ids by ascending distance
    fn search(
        &self,
        mut query: DenseVector,
        _similarity: Similarity,
        k: usize,
    ) -> Result<Vec<PointId>, DbError> {
        if k == 0 {
            return Ok(Vec::new());
        }

        if query.len() != self.data_dimension {
            return Err(DbError::InvalidDimension {
                expected: self.data_dimension,
                got: query.len(),
            });
        }

        let entry = match self.pick_entry() {
            Some(id) => id,
            None => return Ok(Vec::new()),
        };

        self.normalize_if_cosine(&mut query);

        let mut ep = entry;
        let current_max_level = self
            .index
            .nodes
            .get(&entry)
            .map(|n| n.level as usize)
            .unwrap_or(0);
        if current_max_level > 0 {
            for level in (1..=current_max_level).rev() {
                ep = self.greedy_search_layer(ep, level, &query)?;
            }
        }
        let ef0 = max(self.ef, k);
        let mut w = self.search_layer_for_insert(ep, 0, &query, ef0)?;
        w.truncate(k);
        let result: Vec<Uuid> = w.into_iter().map(|(id, _)| id).collect();
        Ok(result)
    }
}

impl HnswIndex {
    /// Full rebuild from surviving (non-deleted) vectors currently in-memory.
    /// Gathers all non-deleted vectors from the cache, clears the graph, and reinserts.
    pub fn rebuild_full(&mut self) -> Result<(), DbError> {
        let ids: Vec<PointId> = self
            .index
            .nodes
            .iter()
            .filter(|(_, n)| !n.deleted)
            .map(|(id, _)| *id)
            .collect();
        let mut points: Vec<IndexedVector> = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(vec) = self.cache.get(&id) {
                points.push(IndexedVector {
                    id,
                    vector: vec.clone(),
                });
            } else {
                continue;
            }
        }
        self.index.nodes.clear();
        for layer in &mut self.index.points_by_layer {
            layer.clear();
        }
        self.index.entry_point = None;
        self.cache.clear();
        for iv in points {
            self.insert(iv)?;
        }
        Ok(())
    }

    /// Fraction of deleted nodes among all nodes (0.0 if no nodes)
    pub fn deleted_ratio(&self) -> f32 {
        let total = self.index.nodes.len();
        if total == 0 {
            return 0.0;
        }
        let deleted = self.index.nodes.values().filter(|n| n.deleted).count();
        deleted as f32 / total as f32
    }

    /// alive/deleted counts and level histogram for alive nodes
    pub fn stats(&self) -> HnswStats {
        let mut alive = 0usize;
        let mut deleted = 0usize;
        let mut hist: std::collections::BTreeMap<u8, usize> = std::collections::BTreeMap::new();
        for n in self.index.nodes.values() {
            if n.deleted {
                deleted += 1;
            } else {
                alive += 1;
                *hist.entry(n.level).or_insert(0) += 1;
            }
        }
        HnswStats {
            alive,
            deleted,
            level_histogram: hist,
        }
    }

    /// Normalize vector in-place if cosine similarity is used
    fn normalize_if_cosine(&self, v: &mut [f32]) {
        if let Similarity::Cosine = self.similarity {
            let sum_sq: f32 = v.iter().map(|&x| x * x).sum();
            let norm = sum_sq.sqrt();
            if norm > 0.0 {
                for x in v {
                    *x /= norm;
                }
            }
        }
    }
}

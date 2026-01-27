use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashSet;

use defs::DbError;
use defs::{OrdF32, PointId};

use crate::distance;

use super::index::HnswIndex;

impl HnswIndex {
    /// Greedy search within a fixed layer
    /// - start from `ep` and evaluate neighbors at `level`
    /// - move to a neighbor only if it strictly improves distance
    /// - stop when no improvement and return the last id
    pub(super) fn greedy_search_layer(
        &self,
        ep: PointId,
        level: usize,
        query: &[f32],
    ) -> Result<PointId, DbError> {
        let mut current = ep;
        loop {
            let cur_vec = self.get_vec(current)?;
            let mut best_score = distance(&query.to_vec(), cur_vec, self.similarity);
            let mut best_id = current;

            let empty: &[PointId] = &[];
            let neighbors = self
                .index
                .nodes
                .get(&current)
                .and_then(|n| n.neighbors.get(level))
                .map(|v| v.as_slice())
                .unwrap_or(empty);

            for &n in neighbors {
                if n == current {
                    continue;
                }
                // Skip deleted neighbors
                if let Some(nn) = self.index.nodes.get(&n)
                    && nn.deleted
                {
                    continue;
                }
                let n_vec = self.get_vec(n)?;
                let score = distance(&query.to_vec(), n_vec, self.similarity);
                if score < best_score {
                    best_score = score;
                    best_id = n;
                }
            }

            if best_id == current {
                break;
            }
            current = best_id;
        }
        Ok(current)
    }

    /// Best-first (ef) search used during insertion on a given layer
    /// - maintain candidate queue and working set up to `ef_construction`
    /// - expand the closest candidate; skip deleted nodes
    /// - early-exit if the best candidate is worse than the worst in W when full
    /// - return W as (id, distance) sorted by ascending distance
    pub(super) fn search_layer_for_insert(
        &self,
        ep: PointId,
        level: usize,
        query: &[f32],
        ef_construction: usize,
    ) -> Result<Vec<(PointId, f32)>, DbError> {
        let mut visited: HashSet<PointId> = HashSet::new();

        let mut candidates: BinaryHeap<(Reverse<OrdF32>, PointId)> = BinaryHeap::new();
        let mut w_heap: BinaryHeap<(OrdF32, PointId)> = BinaryHeap::new();

        // Seed with a non-deleted entry point
        let seed = match self.index.nodes.get(&ep) {
            Some(n) if !n.deleted && (n.level as usize) >= level => ep,
            _ => self
                .index
                .nodes
                .iter()
                .filter(|(_, n)| !n.deleted && (n.level as usize) >= level)
                .max_by(|a, b| a.1.level.cmp(&b.1.level).then_with(|| a.0.cmp(b.0)))
                .map(|(id, _)| *id)
                .unwrap_or(ep),
        };

        let ep_score = distance(&query.to_vec(), self.get_vec(seed)?, self.similarity);
        candidates.push((Reverse(OrdF32::new(ep_score)), seed));
        w_heap.push((OrdF32::new(ep_score), seed));
        visited.insert(seed);

        while let Some((Reverse(best_d), current)) = candidates.pop() {
            if w_heap.len() >= ef_construction
                && let Some(&(worst_d, _)) = w_heap.peek()
                && best_d > worst_d
            {
                break;
            }

            let empty: &[PointId] = &[];
            let neighbors = self
                .index
                .nodes
                .get(&current)
                .and_then(|n| n.neighbors.get(level))
                .map(|v| v.as_slice())
                .unwrap_or(empty);

            for &n in neighbors {
                if visited.contains(&n) {
                    continue;
                }
                // Skip deleted neighbors
                if let Some(nn) = self.index.nodes.get(&n)
                    && nn.deleted
                {
                    continue;
                }

                visited.insert(n);
                let score = distance(&query.to_vec(), self.get_vec(n)?, self.similarity);
                let score = OrdF32::new(score);
                candidates.push((Reverse(score), n));
                if w_heap.len() < ef_construction {
                    w_heap.push((score, n));
                } else if let Some(&(worst_d, _)) = w_heap.peek()
                    && score < worst_d
                {
                    w_heap.pop();
                    w_heap.push((score, n));
                }
            }
        }

        let mut w: Vec<(PointId, f32)> = w_heap
            .into_iter()
            .map(|(d, id)| (id, d.into_inner()))
            .collect();
        w.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        Ok(w)
    }

    /// Diversity-based neighbor selection (heuristic pruning)
    /// - sort candidates by distance-to-new ascending
    /// - accept a candidate unless it is dominated by an accepted one
    /// - return up to `m` ids
    pub(super) fn select_neighbors_heuristic(
        &self,
        candidates: &[(PointId, f32)],
        m: usize,
    ) -> Result<Vec<PointId>, DbError> {
        if candidates.is_empty() || m == 0 {
            return Ok(Vec::new());
        }
        let mut sorted = candidates.to_vec();
        sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let mut result: Vec<PointId> = Vec::with_capacity(m);

        'outer: for &(cand_id, cand_dist_to_q) in &sorted {
            let cand_vec = self.get_vec(cand_id)?;
            for &r_id in &result {
                let r_vec = self.get_vec(r_id)?;
                let cand_to_r = distance(cand_vec, r_vec, self.similarity);
                if cand_to_r < cand_dist_to_q {
                    continue 'outer;
                }
            }
            result.push(cand_id);
            if result.len() >= m {
                break;
            }
        }

        Ok(result)
    }

    /// Connect new node `p` with `neighbors` on `level`
    /// - ensure level storage exists
    /// - merge and prune neighbor lists for `p` and each neighbor (cap by `m`/`M0`)
    /// - skip linking into deleted nodes
    pub(super) fn connect_bidirectional(
        &mut self,
        p: PointId,
        neighbors: &[PointId],
        level: usize,
        m: usize,
    ) -> Result<(), DbError> {
        self.merge_and_prune(p, level, neighbors, m)?;

        for &n in neighbors {
            if n == p {
                continue;
            }
            self.merge_and_prune(n, level, &[p], m)?;
        }
        Ok(())
    }
}

impl HnswIndex {
    /// Pick a non-deleted entry point
    pub(super) fn pick_entry(&self) -> Option<PointId> {
        if let Some(ep) = self.index.entry_point
            && self.index.nodes.get(&ep).is_some_and(|n| !n.deleted)
        {
            return Some(ep);
        }
        self.index
            .nodes
            .iter()
            .filter(|(_, n)| !n.deleted)
            .max_by(|a, b| a.1.level.cmp(&b.1.level).then_with(|| a.0.cmp(b.0)))
            .map(|(id, _)| *id)
    }

    /// Ensure `node.neighbors[level]` exists
    fn ensure_level(&mut self, id: PointId, level: usize) {
        if let Some(node) = self.index.nodes.get(&id)
            && node.neighbors.len() > level
        {
            return;
        }

        let node = self.index.nodes.get_mut(&id).expect("node must exist");
        if node.neighbors.len() <= level {
            node.neighbors.resize(level + 1, Vec::new());
        }
    }

    /// Merge existing neighbors at `level` with `to_add`, then score by distance to `center`.
    /// Sort ascending by distance and cap the final list to `cap` entries (no diversity heuristic).
    /// Write the pruned list back to `center.neighbors[level]`.
    fn merge_and_prune(
        &mut self,
        center: PointId,
        level: usize,
        to_add: &[PointId],
        cap: usize,
    ) -> Result<(), DbError> {
        self.ensure_level(center, level);

        let mut merged: Vec<PointId> = {
            let center_node = self.index.nodes.get_mut(&center).unwrap();
            std::mem::take(&mut center_node.neighbors[level])
        };

        let mut seen: HashSet<PointId> = merged.iter().copied().collect();

        for &n in to_add {
            if n == center {
                continue;
            }
            if !seen.insert(n) {
                continue;
            }
            if let Some(nn) = self.index.nodes.get(&n)
                && nn.deleted
            {
                continue;
            }
            merged.push(n);
        }

        let center_vec = self.get_vec(center)?;
        let mut scored: Vec<(PointId, f32)> = Vec::with_capacity(merged.len());

        for nid in merged {
            let d = distance(center_vec, self.get_vec(nid)?, self.similarity);
            scored.push((nid, d));
        }
        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        scored.truncate(cap);
        let new_list: Vec<PointId> = scored.into_iter().map(|(nid, _)| nid).collect();

        {
            let center_node = self.index.nodes.get_mut(&center).unwrap();
            center_node.neighbors[level] = new_list;
        }
        Ok(())
    }
}

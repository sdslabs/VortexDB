use std::collections::HashMap;

use defs::PointId;
use rand::Rng;

// Compact storage for layered points and adjacency used by `HnswIndex`.
pub struct PointIndexation {
    // Max connections per point per layer (M)
    pub max_connections: usize,
    // Max edges per node on layer 0 (often 2*M)
    pub max_connections_0: usize,
    // Maximum number of layers
    pub max_layer: usize,
    // Points per layer; each inner Vec holds the PointId(s)
    pub points_by_layer: Vec<Vec<PointId>>,
    // Per-node, per-level neighbor lists (bounded by M/M0)
    pub nodes: HashMap<PointId, Node>,
    // Optional entry point used for searches/insertions
    pub entry_point: Option<PointId>,
    // Level generator used to sample random levels
    pub level_generator: LevelGenerator,
}

// Node with highest level and per-level neighbor lists
pub struct Node {
    pub id: PointId,
    // Highest level (0-based; level 0 is the base layer)
    pub level: u8,
    // neighbors[level] -> neighbor PointIds at that level
    pub neighbors: Vec<Vec<PointId>>,
    // Soft-delete flag (skipped by traversals)
    pub deleted: bool,
}

// Level sampling parameters
pub struct LevelGenerator {
    // 1 / ln(M)
    pub level_scale: f64,
}

impl LevelGenerator {
    pub fn from_m(m: usize) -> Self {
        assert!(m >= 2, "LevelGenerator::from_m: m must be >= 2");
        let level_scale = 1.0 / (m as f64).ln();
        Self { level_scale }
    }

    /// Sample a level `L` from an exponential tail: P(L ≥ l) ≈ exp(-l / ln M).
    /// Uses inverse transform: L = floor(-ln(U) * (1/ln M)), capped to `max_layer - 1`.
    pub fn sample_level<R: Rng>(&self, rng: &mut R, max_layer: usize) -> u8 {
        let mut u: f64 = rng.random();
        if u <= 0.0 {
            u = f64::EPSILON;
        }
        if u >= 1.0 {
            u = 1.0 - f64::EPSILON;
        }

        let raw = (-u.ln()) * self.level_scale;
        let l = raw.floor() as usize;
        let capped = l.min(max_layer.saturating_sub(1));
        capped as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HnswStats {
    pub alive: usize,
    pub deleted: usize,
    pub level_histogram: std::collections::BTreeMap<u8, usize>,
}

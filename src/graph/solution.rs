use std::collections::HashSet;

use crate::graph::{Edge, EdgeSet};

/// A complete solution to the puzzle
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Solution {
    /// The edges in this solution (order doesn't matter for equality)
    edges: HashSet<Edge>,
}

impl Solution {
    pub fn new() -> Self {
        Solution {
            edges: HashSet::new(),
        }
    }

    pub fn from_edge_set(edge_set: &EdgeSet) -> Self {
        Solution {
            edges: edge_set.edges_in_order().iter().copied().collect(),
        }
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.insert(edge);
    }

    pub fn contains(&self, edge: &Edge) -> bool {
        self.edges.contains(edge)
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Check if this solution matches another (same edges, any order)
    pub fn matches(&self, other: &Solution) -> bool {
        self.edges == other.edges
    }

    /// Get a canonical string representation for serialization/comparison
    /// Format: "0-1,1-2,2-3" (sorted)
    pub fn canonical_string(&self) -> String {
        let mut edges: Vec<_> = self.edges.iter().collect();
        edges.sort_by_key(|e| (e.from.0, e.to.0));
        edges
            .iter()
            .map(|e| format!("{}-{}", e.from.0, e.to.0))
            .collect::<Vec<_>>()
            .join(",")
    }
}

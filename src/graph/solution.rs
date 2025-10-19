use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use crate::graph::{Edge, EdgeSet};

/// A complete solution to the puzzle
/// Two solutions are equal if they contain the same edges, regardless of order
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Solution {
    /// The edges in this solution (order doesn't matter for equality)
    edges: HashSet<Edge>,
}

// Manual Hash implementation: sort edges to get deterministic hash
impl Hash for Solution {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Sort edges to ensure consistent hashing
        let mut edges: Vec<_> = self.edges.iter().collect();
        edges.sort_unstable_by_key(|e| (e.from, e.to));
        
        // Hash the sorted edges
        for edge in edges {
            edge.hash(state);
        }
    }
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
        // This is redundant with PartialEq, but kept for clarity
        self == other
    }
    
    /// Get all edges in this solution
    pub fn edges(&self) -> &HashSet<Edge> {
        &self.edges
    }
    
    /// Check if solution is empty
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
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

impl Default for Solution {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::NodeId;
    
    #[test]
    fn test_solution_equality_order_independent() {
        let mut sol1 = Solution::new();
        sol1.add_edge(Edge::new(NodeId(0), NodeId(1)));
        sol1.add_edge(Edge::new(NodeId(1), NodeId(2)));
        sol1.add_edge(Edge::new(NodeId(2), NodeId(0)));
        
        let mut sol2 = Solution::new();
        sol2.add_edge(Edge::new(NodeId(2), NodeId(0))); // Different order
        sol2.add_edge(Edge::new(NodeId(0), NodeId(1)));
        sol2.add_edge(Edge::new(NodeId(1), NodeId(2)));
        
        assert_eq!(sol1, sol2, "Solutions with same edges in different order should be equal");
    }
    
    #[test]
    fn test_solution_hashing() {
        use std::collections::HashSet;
        
        let mut sol1 = Solution::new();
        sol1.add_edge(Edge::new(NodeId(0), NodeId(1)));
        sol1.add_edge(Edge::new(NodeId(1), NodeId(2)));
        
        let mut sol2 = Solution::new();
        sol2.add_edge(Edge::new(NodeId(1), NodeId(2))); // Same edges, different order
        sol2.add_edge(Edge::new(NodeId(0), NodeId(1)));
        
        let mut solutions = HashSet::new();
        solutions.insert(sol1);
        
        // Should recognize sol2 as duplicate
        assert!(solutions.contains(&sol2), "HashSet should find equivalent solution");
        assert_eq!(solutions.len(), 1, "Should only have one unique solution");
    }
    
    #[test]
    fn test_solution_checking() {
        // Create known solutions
        let mut known = HashSet::new();
        
        let mut sol1 = Solution::new();
        sol1.add_edge(Edge::new(NodeId(0), NodeId(1)));
        sol1.add_edge(Edge::new(NodeId(1), NodeId(2)));
        known.insert(sol1);
        
        // Player draws same solution in different order
        let mut player_solution = Solution::new();
        player_solution.add_edge(Edge::new(NodeId(1), NodeId(2)));
        player_solution.add_edge(Edge::new(NodeId(0), NodeId(1)));
        
        assert!(known.contains(&player_solution), "Should recognize player found known solution");
        
        // Player draws different solution
        let mut new_solution = Solution::new();
        new_solution.add_edge(Edge::new(NodeId(0), NodeId(3)));
        new_solution.add_edge(Edge::new(NodeId(3), NodeId(1)));
        
        assert!(!known.contains(&new_solution), "Should recognize this is a new solution");
    }
}

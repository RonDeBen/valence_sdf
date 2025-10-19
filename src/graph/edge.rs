use super::kings_graph::NodeId;

use std::collections::HashSet;

/// An edge between two nodes
/// Invariant: always stored in canonical form with from <= to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
}

impl Edge {
    /// Create a new edge, automatically ordering nodes
    pub fn new(a: NodeId, b: NodeId) -> Self {
        if a <= b {
            Edge { from: a, to: b }
        } else {
            Edge { from: b, to: a }
        }
    }

    /// Check if this edge contains a given node
    pub fn contains_node(&self, node: NodeId) -> bool {
        self.from == node || self.to == node
    }

    /// Get the other node in the edge
    pub fn other_node(&self, node: NodeId) -> Option<NodeId> {
        if self.from == node {
            Some(self.to)
        } else if self.to == node {
            Some(self.from)
        } else {
            None
        }
    }
}

/// A set of edges with efficient lookup
/// Maintains both the set of edges and an ordered list of edges in draw order
#[derive(Debug, Clone)]
pub struct EdgeSet {
    /// Set for O(1) edge existence checks
    edges: HashSet<Edge>,
    /// Ordered list of edges in the order they were drawn
    draw_order: Vec<Edge>,
}

impl EdgeSet {
    pub fn new() -> Self {
        EdgeSet {
            edges: HashSet::new(),
            draw_order: Vec::new(),
        }
    }

    /// Add an edge to the set
    /// Returns true if the edge was newly inserted, false if it already existed
    pub fn add(&mut self, edge: Edge) -> bool {
        if self.edges.insert(edge) {
            self.draw_order.push(edge);
            true
        } else {
            false
        }
    }

    /// Check if an edge exists in the set
    pub fn contains(&self, edge: &Edge) -> bool {
        self.edges.contains(edge)
    }

    /// Remove the last edge added
    pub fn pop(&mut self) -> Option<Edge> {
        if let Some(edge) = self.draw_order.pop() {
            self.edges.remove(&edge);
            Some(edge)
        } else {
            None
        }
    }

    /// Get the number of edges
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Check if the edge set is empty
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Get edges in draw order
    pub fn edges_in_order(&self) -> &[Edge] {
        &self.draw_order
    }

    /// Get the last edge added, if any
    pub fn last(&self) -> Option<Edge> {
        self.draw_order.last().copied()
    }

    /// Clear all edges
    pub fn clear(&mut self) {
        self.edges.clear();
        self.draw_order.clear();
    }

    /// Count how many edges are incident to a given node
    pub fn degree(&self, node: NodeId) -> usize {
        self.edges
            .iter()
            .filter(|edge| edge.contains_node(node))
            .count()
    }
}

impl Default for EdgeSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_canonical_form() {
        let e1 = Edge::new(NodeId(1), NodeId(3));
        let e2 = Edge::new(NodeId(3), NodeId(1));

        assert_eq!(e1, e2, "Edges should be equal regardless of order");
        assert_eq!(e1.from, NodeId(1));
        assert_eq!(e1.to, NodeId(3));
    }

    #[test]
    fn test_edge_contains_node() {
        let edge = Edge::new(NodeId(1), NodeId(3));

        assert!(edge.contains_node(NodeId(1)));
        assert!(edge.contains_node(NodeId(3)));
        assert!(!edge.contains_node(NodeId(2)));
    }

    #[test]
    fn test_edge_set_basic_ops() {
        let mut set = EdgeSet::new();

        let e1 = Edge::new(NodeId(0), NodeId(1));
        let e2 = Edge::new(NodeId(1), NodeId(2));

        assert!(set.add(e1));
        assert!(set.add(e2));
        assert!(!set.add(e1), "Adding duplicate should return false");

        assert_eq!(set.len(), 2);
        assert!(set.contains(&e1));
        assert!(set.contains(&e2));
    }

    #[test]
    fn test_edge_set_draw_order() {
        let mut set = EdgeSet::new();

        let e1 = Edge::new(NodeId(0), NodeId(1));
        let e2 = Edge::new(NodeId(1), NodeId(2));
        let e3 = Edge::new(NodeId(2), NodeId(3));

        set.add(e1);
        set.add(e2);
        set.add(e3);

        let order = set.edges_in_order();
        assert_eq!(order.len(), 3);
        assert_eq!(order[0], e1);
        assert_eq!(order[1], e2);
        assert_eq!(order[2], e3);
    }

    #[test]
    fn test_edge_set_pop() {
        let mut set = EdgeSet::new();

        let e1 = Edge::new(NodeId(0), NodeId(1));
        let e2 = Edge::new(NodeId(1), NodeId(2));

        set.add(e1);
        set.add(e2);

        assert_eq!(set.pop(), Some(e2));
        assert_eq!(set.len(), 1);
        assert!(!set.contains(&e2));
        assert!(set.contains(&e1));

        assert_eq!(set.pop(), Some(e1));
        assert_eq!(set.len(), 0);
        assert!(set.is_empty());
    }

    #[test]
    fn test_edge_set_degree() {
        let mut set = EdgeSet::new();

        // Not a graph we would make in our 3x3 King's Graph
        set.add(Edge::new(NodeId(0), NodeId(1)));
        set.add(Edge::new(NodeId(0), NodeId(2)));
        set.add(Edge::new(NodeId(0), NodeId(3)));

        assert_eq!(set.degree(NodeId(0)), 3);
        assert_eq!(set.degree(NodeId(1)), 1);
        assert_eq!(set.degree(NodeId(2)), 1);
        assert_eq!(set.degree(NodeId(3)), 1);
        assert_eq!(set.degree(NodeId(4)), 0);
    }
}

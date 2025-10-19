use std::fmt;

/// Node identifier (0-8 for 3x3 grid)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub usize);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl NodeId {
    pub const fn new(id: usize) -> Self {
        NodeId(id)
    }

    pub const fn index(&self) -> usize {
        self.0
    }

    /// Check if this is a valid node ID for 3x3 grid
    pub const fn is_valid(&self) -> bool {
        self.0 < 9
    }
}

/// Grid position (row, col) both in range [0, 2]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub row: usize,
    pub col: usize,
}

impl GridPos {
    pub const fn new(row: usize, col: usize) -> Self {
        GridPos { row, col }
    }

    /// Check if two positions are adjacent (king's move)
    pub fn is_adjacent(&self, other: &GridPos) -> bool {
        if self == other {
            return false;
        }

        let row_diff = (self.row as i32 - other.row as i32).abs();
        let col_diff = (self.col as i32 - other.col as i32).abs();

        row_diff <= 1 && col_diff <= 1
    }

    /// Convert grid position to node ID (0-8)
    /// Layout:
    /// 0 1 2
    /// 3 4 5
    /// 6 7 8
    pub const fn to_node_id(&self) -> NodeId {
        let node_id = NodeId(self.row * 3 + self.col);
        assert!(node_id.is_valid());
        node_id
    }

    /// Convert node ID to grid position
    pub const fn from_node_id(node: NodeId) -> Self {
        assert!(node.is_valid());
        GridPos {
            row: node.0 / 3,
            col: node.0 % 3,
        }
    }
}

/// King's graph structure for a 3x3 grid
/// This represents ONLY the adjacency relationships, not valences
#[derive(Debug, Clone)]
pub struct KingsGraph {
    adjacency: Vec<Vec<NodeId>>,
}

impl KingsGraph {
    /// Create a new 3x3 king's graph
    pub fn new_3x3() -> Self {
        let mut adjacency = vec![Vec::new(); 9];

        for i in 0..9 {
            let node = NodeId(i);
            let pos = GridPos::from_node_id(node);

            for j in 0..9 {
                if i == j {
                    continue;
                }

                let other = NodeId(j);
                let other_pos = GridPos::from_node_id(other);

                if pos.is_adjacent(&other_pos) {
                    adjacency[i].push(other);
                }
            }
        }

        KingsGraph { adjacency }
    }

    /// Check if two nodes are adjacent
    pub fn are_adjacent(&self, a: NodeId, b: NodeId) -> bool {
        if a.index() >= self.adjacency.len() || b.index() >= self.adjacency.len() {
            return false;
        }

        self.adjacency[a.index()].contains(&b)
    }

    /// Get all neighbors of a node
    pub fn neighbors(&self, node: NodeId) -> &[NodeId] {
        &self.adjacency[node.index()]
    }
}

impl Default for KingsGraph {
    fn default() -> Self {
        Self::new_3x3()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_pos_adjacency() {
        let center = GridPos::new(1, 1);

        // All 8 surrounding positions should be adjacent
        assert!(center.is_adjacent(&GridPos::new(0, 0)));
        assert!(center.is_adjacent(&GridPos::new(0, 1)));
        assert!(center.is_adjacent(&GridPos::new(0, 2)));
        assert!(center.is_adjacent(&GridPos::new(1, 0)));
        assert!(center.is_adjacent(&GridPos::new(1, 2)));
        assert!(center.is_adjacent(&GridPos::new(2, 0)));
        assert!(center.is_adjacent(&GridPos::new(2, 1)));
        assert!(center.is_adjacent(&GridPos::new(2, 2)));

        // Not adjacent to itself
        assert!(!center.is_adjacent(&center));
    }

    #[test]
    fn test_node_id_conversion() {
        assert_eq!(GridPos::new(0, 0).to_node_id(), NodeId(0));
        assert_eq!(GridPos::new(0, 1).to_node_id(), NodeId(1));
        assert_eq!(GridPos::new(0, 2).to_node_id(), NodeId(2));
        assert_eq!(GridPos::new(1, 0).to_node_id(), NodeId(3));
        assert_eq!(GridPos::new(1, 1).to_node_id(), NodeId(4));
        assert_eq!(GridPos::new(1, 2).to_node_id(), NodeId(5));
        assert_eq!(GridPos::new(2, 0).to_node_id(), NodeId(6));
        assert_eq!(GridPos::new(2, 1).to_node_id(), NodeId(7));
        assert_eq!(GridPos::new(2, 2).to_node_id(), NodeId(8));

        // Test round-trip
        for i in 0..9 {
            let node = NodeId(i);
            let pos = GridPos::from_node_id(node);
            assert_eq!(pos.to_node_id(), node);
        }
    }

    #[test]
    fn test_kings_graph_adjacency() {
        let graph = KingsGraph::new_3x3();

        // Center node (4) should have 8 neighbors
        assert_eq!(graph.neighbors(NodeId(4)).len(), 8);

        // Corner node (0) should have 3 neighbors
        assert_eq!(graph.neighbors(NodeId(0)).len(), 3);
        assert!(graph.are_adjacent(NodeId(0), NodeId(1)));
        assert!(graph.are_adjacent(NodeId(0), NodeId(3)));
        assert!(graph.are_adjacent(NodeId(0), NodeId(4)));

        // Edge node (1) should have 5 neighbors
        assert_eq!(graph.neighbors(NodeId(1)).len(), 5);

        // Symmetry: if A is adjacent to B, then B is adjacent to A
        for i in 0..9 {
            for j in 0..9 {
                let a_to_b = graph.are_adjacent(NodeId(i), NodeId(j));
                let b_to_a = graph.are_adjacent(NodeId(j), NodeId(i));
                assert_eq!(a_to_b, b_to_a, "Adjacency should be symmetric");
            }
        }
    }
}

use super::edge::{Edge, EdgeSet};
use super::kings_graph::{KingsGraph, NodeId};
use super::valences::Valences;
use std::fmt;

/// Error types for move validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    NodeHasNoValence(NodeId),
    NodesNotAdjacent(NodeId, NodeId),
    EdgeAlreadyExists(Edge),
    CannotAddValenceOne(NodeId),
    SameNodeTwice(NodeId),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::NodeHasNoValence(n) => {
                write!(f, "Node {} has no remaining valence", n)
            }
            ValidationError::NodesNotAdjacent(a, b) => {
                write!(f, "Nodes {} and {} are not adjacent", a, b)
            }
            ValidationError::EdgeAlreadyExists(e) => {
                write!(f, "Edge {}-{} already exists", e.from, e.to)
            }
            ValidationError::CannotAddValenceOne(n) => {
                write!(f, "Cannot add valence-1 node {} (not the last edge)", n)
            }
            ValidationError::SameNodeTwice(n) => write!(f, "Cannot add node {} twice in a row", n),
        }
    }
}

/// Result of attempting to add a node
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveResult {
    EdgeAdded(Edge),
    FirstNode(NodeId),
    Invalid(ValidationError),
    PuzzleComplete,
}

/// Game state for the valence puzzle
#[derive(Debug, Clone)]
pub struct GameState {
    /// The underlying graph structure (adjacency only)
    graph: KingsGraph,

    /// The puzzle configuration (never changes during play)
    puzzle_valences: Valences,

    /// Current valence state (changes as edges are drawn)
    current_valences: Valences,

    /// Set of edges that have been drawn
    edges: EdgeSet,

    /// Current trail being drawn (nodes in order)
    /// Note: This is NOT the same as edges! The trail includes the starting node
    /// Example: trail [0, 1, 4] represents edges (0-1) and (1-4)
    current_trail: Vec<NodeId>,
}

impl GameState {
    /// Create a new game with given puzzle valences
    pub fn new(puzzle_valences: Valences) -> Self {
        GameState {
            graph: KingsGraph::default(),
            puzzle_valences: puzzle_valences.clone(),
            current_valences: puzzle_valences,
            edges: EdgeSet::new(),
            current_trail: Vec::new(),
        }
    }

    /// Get current valence of a node
    pub fn valence(&self, node: NodeId) -> usize {
        self.current_valences.get(node)
    }

    /// Get all current valences (for display)
    pub fn valences(&self) -> &Valences {
        &self.current_valences
    }

    /// Get the puzzle configuration
    pub fn puzzle_valences(&self) -> &Valences {
        &self.puzzle_valences
    }

    /// Get the current trail of nodes
    pub fn current_trail(&self) -> &[NodeId] {
        &self.current_trail
    }

    /// Get all edges that have been drawn
    pub fn edges(&self) -> &EdgeSet {
        &self.edges
    }

    /// Get total remaining valence
    pub fn total_remaining_valence(&self) -> usize {
        self.current_valences.total()
    }

    /// Check if we're at the last edge (total valence == 2)
    pub fn is_last_edge(&self) -> bool {
        self.total_remaining_valence() == 2
    }

    /// Check if the puzzle is complete (all valences are 0)
    pub fn is_complete(&self) -> bool {
        self.current_valences.all_zero()
    }

    /// Validate if a node can be added to the current trail
    pub fn can_add_node(&self, node: NodeId) -> Result<(), ValidationError> {
        // Check if node has valence
        if self.valence(node) == 0 {
            return Err(ValidationError::NodeHasNoValence(node));
        }

        // If this is the first node, it's always valid
        if self.current_trail.is_empty() {
            return Ok(());
        }

        let last_node = *self.current_trail.last().unwrap();

        // Can't add the same node twice in a row
        if node == last_node {
            return Err(ValidationError::SameNodeTwice(node));
        }

        // Nodes must be adjacent
        if !self.graph.are_adjacent(node, last_node) {
            return Err(ValidationError::NodesNotAdjacent(last_node, node));
        }

        // Edge must not already exist
        let edge = Edge::new(node, last_node);
        if self.edges.contains(&edge) {
            return Err(ValidationError::EdgeAlreadyExists(edge));
        }

        // Can't add a valence-1 node unless it's the last edge needed
        if self.valence(node) == 1 && !self.is_last_edge() {
            return Err(ValidationError::CannotAddValenceOne(node));
        }

        Ok(())
    }

    /// Add a node to the current trail
    pub fn add_node(&mut self, node: NodeId) -> MoveResult {
        // Validate the move
        if let Err(e) = self.can_add_node(node) {
            return MoveResult::Invalid(e);
        }

        // If this is the first node, just add it to the trail
        if self.current_trail.is_empty() {
            self.current_trail.push(node);
            return MoveResult::FirstNode(node);
        }

        // Add the edge and update valences
        let last_node = *self.current_trail.last().unwrap();
        let edge = Edge::new(node, last_node);
        self.edges.add(edge);

        self.current_valences.decrement(node);
        self.current_valences.decrement(last_node);

        self.current_trail.push(node);

        // Check if puzzle is complete
        if self.is_complete() {
            MoveResult::PuzzleComplete
        } else {
            MoveResult::EdgeAdded(edge)
        }
    }

    /// Remove the last node from the trail (undo)
    pub fn pop_node(&mut self) -> Option<NodeId> {
        if self.current_trail.len() <= 1 {
            // If there's only one node or none, just clear the trail
            self.current_trail.clear();
            return None;
        }

        let node = self.current_trail.pop()?;
        let prev_node = *self.current_trail.last().unwrap();

        // Remove the edge and restore valences
        if let Some(_edge) = self.edges.pop() {
            self.current_valences.increment(node);
            self.current_valences.increment(prev_node);
        }

        Some(node)
    }

    /// Reset to the initial puzzle state
    pub fn reset(&mut self) {
        self.current_valences = self.puzzle_valences.clone();
        self.edges.clear();
        self.current_trail.clear();
    }

    /// Get all nodes that are currently valid to add
    pub fn valid_next_nodes(&self) -> Vec<NodeId> {
        (0..9)
            .map(NodeId)
            .filter(|&node| self.can_add_node(node).is_ok())
            .collect()
    }

    /// Get all nodes that should "flee" (cannot be added)
    pub fn nodes_that_should_flee(&self) -> Vec<NodeId> {
        if self.current_trail.is_empty() {
            return Vec::new();
        }

        let last_node = *self.current_trail.last().unwrap();

        (0..9)
            .map(NodeId)
            .filter(|&node| node != last_node && self.can_add_node(node).is_err())
            .collect()
    }

    /// Count available edges for a node (for degenerate detection)
    fn count_available_edges(&self, node: NodeId) -> usize {
        self.graph
            .neighbors(node)
            .iter()
            .filter(|&&neighbor| {
                let edge = Edge::new(node, neighbor);
                !self.edges.contains(&edge) && self.valence(neighbor) > 0
            })
            .count()
    }

    /// Check if the puzzle is in a degenerate state (unsolvable)
    pub fn is_degenerate(&self) -> bool {
        // Check if any node can't satisfy its remaining valence
        for i in 0..9 {
            let node = NodeId(i);
            let valence = self.valence(node);

            if valence == 0 {
                continue;
            }

            let available = self.count_available_edges(node);

            if valence > available {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_game() {
        // Triangle: nodes 0, 1, 3
        let valences = Valences::new(vec![2, 2, 0, 2, 0, 0, 0, 0, 0]);
        let mut state = GameState::new(valences);

        assert_eq!(state.add_node(NodeId(0)), MoveResult::FirstNode(NodeId(0)));
        assert!(matches!(
            state.add_node(NodeId(1)),
            MoveResult::EdgeAdded(_)
        ));
        assert!(matches!(
            state.add_node(NodeId(3)),
            MoveResult::EdgeAdded(_)
        ));
        assert_eq!(state.add_node(NodeId(0)), MoveResult::PuzzleComplete);

        assert!(state.is_complete());
    }

    #[test]
    fn test_reset() {
        let valences = Valences::new(vec![1, 1, 0, 0, 0, 0, 0, 0, 0]);
        let mut state = GameState::new(valences.clone());

        state.add_node(NodeId(0));
        state.add_node(NodeId(1));

        state.reset();

        assert_eq!(state.valences(), &valences);
        assert!(state.current_trail().is_empty());
        assert!(state.edges().is_empty());
    }
}

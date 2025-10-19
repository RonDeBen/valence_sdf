// game/session.rs

use crate::graph::*;
use bevy::prelude::Resource;
use std::collections::HashSet;

/// A game session - manages one puzzle instance
#[derive(Debug, Clone, Resource)]
pub struct PuzzleSession {
    /// The core game state
    state: GameState,
    /// Solutions the player has found for this puzzle
    found_solutions: HashSet<Solution>,
    /// Total number of solutions for this puzzle (if known)
    total_solutions: usize,
}

impl PuzzleSession {
    /// Create a new session with a puzzle
    pub fn new(puzzle_valences: Valences, total_solutions: usize) -> Self {
        PuzzleSession {
            state: GameState::new(puzzle_valences),
            found_solutions: HashSet::new(),
            total_solutions,
        }
    }

    // === Query Methods (for Bevy systems to read state) ===

    /// Is the current puzzle complete?
    pub fn is_complete(&self) -> bool {
        self.state.is_complete()
    }

    /// Get current valences (for visual display)
    pub fn current_valences(&self) -> &Valences {
        self.state.valences()
    }

    /// Get the puzzle configuration
    pub fn puzzle_valences(&self) -> &Valences {
        self.state.puzzle_valences()
    }

    /// Get current trail of nodes
    pub fn current_trail(&self) -> &[NodeId] {
        self.state.current_trail()
    }

    /// Get all drawn edges
    pub fn edges(&self) -> &EdgeSet {
        self.state.edges()
    }

    /// Which nodes should flee from user input?
    pub fn nodes_to_flee(&self) -> Vec<NodeId> {
        self.state.nodes_that_should_flee()
    }

    /// Which nodes are valid to add?
    pub fn valid_nodes(&self) -> Vec<NodeId> {
        self.state.valid_next_nodes()
    }

    /// Check if a specific node can be added
    pub fn can_add_node(&self, node: NodeId) -> Result<(), ValidationError> {
        self.state.can_add_node(node)
    }

    /// Get progress info
    pub fn progress(&self) -> ProgressInfo {
        ProgressInfo {
            solutions_found: self.found_solutions.len(),
            total_solutions: Some(self.total_solutions),
            current_edges: self.state.edges().len(),
        }
    }

    /// Check if current state is degenerate (unsolvable)
    pub fn is_degenerate(&self) -> bool {
        self.state.is_degenerate()
    }

    /// Has this exact solution been found before?
    pub fn is_solution_known(&self, solution: &Solution) -> bool {
        self.found_solutions.contains(solution)
    }

    /// Get all found solutions
    pub fn found_solutions(&self) -> &HashSet<Solution> {
        &self.found_solutions
    }

    // === Mutation Methods (for handling user input) ===

    /// Try to add a node to the current trail
    pub fn add_node(&mut self, node: NodeId) -> SessionResult {
        match self.state.add_node(node) {
            MoveResult::PuzzleComplete => {
                let solution = Solution::from_edge_set(self.state.edges());
                let is_new = !self.is_solution_known(&solution);

                if is_new {
                    self.found_solutions.insert(solution.clone());
                }

                SessionResult::Complete { solution, is_new }
            }
            MoveResult::EdgeAdded(edge) => SessionResult::EdgeAdded(edge),
            MoveResult::FirstNode(node) => SessionResult::FirstNode(node),
            MoveResult::Invalid(err) => SessionResult::Invalid(err),
        }
    }

    /// Undo last move
    pub fn undo(&mut self) -> Option<NodeId> {
        self.state.pop_node()
    }

    /// Reset the current attempt (keeps found solutions)
    pub fn reset(&mut self) {
        self.state.reset();
    }

    /// Start a completely new puzzle (clears found solutions)
    pub fn new_puzzle(&mut self, puzzle_valences: Valences, total_solutions: usize) {
        self.state = GameState::new(puzzle_valences);
        self.found_solutions.clear();
        self.total_solutions = total_solutions;
    }
}

/// Result of a session action
#[derive(Debug, Clone)]
pub enum SessionResult {
    /// First node was placed (no edge yet)
    FirstNode(NodeId),
    /// An edge was added successfully
    EdgeAdded(Edge),
    /// Puzzle was completed
    Complete { solution: Solution, is_new: bool },
    /// Move was invalid
    Invalid(ValidationError),
}

/// Progress information for UI display
#[derive(Debug, Clone, Copy)]
pub struct ProgressInfo {
    pub solutions_found: usize,
    pub total_solutions: Option<usize>,
    pub current_edges: usize,
}

impl ProgressInfo {
    /// Format as a string like "2/5 solutions found"
    pub fn display_string(&self) -> String {
        match self.total_solutions {
            Some(total) => format!("{}/{} solutions", self.solutions_found, total),
            None => format!("{} solutions", self.solutions_found),
        }
    }

    /// Check if all solutions have been found
    pub fn is_complete(&self) -> bool {
        self.total_solutions
            .map_or(false, |total| self.solutions_found >= total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_tracks_solutions() {
        let valences = Valences::new(vec![2, 2, 0, 2, 0, 0, 0, 0, 0]);
        let mut session = PuzzleSession::new(valences, 1);

        // Complete the puzzle
        session.add_node(NodeId(0));
        session.add_node(NodeId(1));
        session.add_node(NodeId(3));
        let result = session.add_node(NodeId(0));

        // Check it was tracked as new
        assert!(matches!(
            result,
            SessionResult::Complete { is_new: true, .. }
        ));
        assert_eq!(session.found_solutions().len(), 1);

        // Reset and do it again
        session.reset();
        session.add_node(NodeId(0));
        session.add_node(NodeId(1));
        session.add_node(NodeId(3));
        let result = session.add_node(NodeId(0));

        // Should recognize it as already found
        assert!(matches!(
            result,
            SessionResult::Complete { is_new: false, .. }
        ));
        assert_eq!(session.found_solutions().len(), 1); // Still only 1 unique solution
    }

    #[test]
    fn test_progress_info() {
        let valences = Valences::new(vec![1, 1, 0, 0, 0, 0, 0, 0, 0]);
        let mut session = PuzzleSession::new(valences, 1);

        let progress = session.progress();
        assert_eq!(progress.solutions_found, 0);
        assert_eq!(progress.display_string(), "0/1 solutions");
        assert!(!progress.is_complete());

        // Complete it
        session.add_node(NodeId(0));
        session.add_node(NodeId(1));

        let progress = session.progress();
        assert_eq!(progress.solutions_found, 1);
        assert!(progress.is_complete());
    }

    #[test]
    fn test_new_puzzle_clears_solutions() {
        let valences1 = Valences::new(vec![1, 1, 0, 0, 0, 0, 0, 0, 0]);
        let mut session = PuzzleSession::new(valences1, 1);

        // Complete first puzzle
        session.add_node(NodeId(0));
        session.add_node(NodeId(1));
        assert_eq!(session.found_solutions().len(), 1);

        // Start new puzzle
        let valences2 = Valences::new(vec![2, 2, 0, 2, 0, 0, 0, 0, 0]);
        session.new_puzzle(valences2, 1);

        // Solutions should be cleared
        assert_eq!(session.found_solutions().len(), 0);
        assert_eq!(session.puzzle_valences().get(NodeId(0)), 2);
    }
}

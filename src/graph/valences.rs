// valences.rs - NEW FILE

use super::kings_graph::NodeId;
use std::fmt;

/// Valence values for all 9 nodes in the grid
/// Always exactly 9 values, indexed by NodeId
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Valences([usize; 9]);

impl Valences {
    /// Create from a Vec (must be length 9)
    pub fn new(values: Vec<usize>) -> Self {
        assert_eq!(values.len(), 9, "Valences must have exactly 9 values");
        let mut arr = [0; 9];
        arr.copy_from_slice(&values);
        Valences(arr)
    }

    /// Create from array directly
    pub fn from_array(values: [usize; 9]) -> Self {
        Valences(values)
    }

    /// All zeros (target state)
    pub const fn zeros() -> Self {
        Valences([0; 9])
    }

    /// Get valence for a specific node
    pub fn get(&self, node: NodeId) -> usize {
        self.0[node.index()]
    }

    /// Set valence for a specific node
    pub fn set(&mut self, node: NodeId, value: usize) {
        self.0[node.index()] = value;
    }

    /// Decrement valence for a node
    pub fn decrement(&mut self, node: NodeId) {
        self.0[node.index()] -= 1;
    }

    /// Increment valence for a node
    pub fn increment(&mut self, node: NodeId) {
        self.0[node.index()] += 1;
    }

    /// Check if all valences are zero
    pub fn all_zero(&self) -> bool {
        self.0.iter().all(|&v| v == 0)
    }

    /// Get nodes with odd valence
    pub fn odd_nodes(&self) -> Vec<NodeId> {
        (0..9)
            .map(NodeId)
            .filter(|&n| self.get(n) % 2 == 1)
            .collect()
    }

    pub fn total(&self) -> usize {
        self.0.iter().sum()
    }
}

impl fmt::Display for Valences {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {} {}", self.0[0], self.0[1], self.0[2])?;
        writeln!(f, "{} {} {}", self.0[3], self.0[4], self.0[5])?;
        write!(f, "{} {} {}", self.0[6], self.0[7], self.0[8])
    }
}

impl From<[usize; 9]> for Valences {
    fn from(arr: [usize; 9]) -> Self {
        Valences(arr)
    }
}

impl From<Vec<usize>> for Valences {
    fn from(vec: Vec<usize>) -> Self {
        Valences::new(vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valences_creation() {
        let v = Valences::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(v.get(NodeId(0)), 1);
        assert_eq!(v.get(NodeId(8)), 9);
    }

    #[test]
    fn test_valences_modification() {
        let mut v = Valences::zeros();
        v.set(NodeId(4), 5);
        assert_eq!(v.get(NodeId(4)), 5);

        v.decrement(NodeId(4));
        assert_eq!(v.get(NodeId(4)), 4);
    }

    #[test]
    fn test_all_zero() {
        let v = Valences::zeros();
        assert!(v.all_zero());

        let mut v2 = Valences::zeros();
        v2.set(NodeId(0), 1);
        assert!(!v2.all_zero());
    }

    #[test]
    fn test_odd_nodes() {
        let v = Valences::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let odd = v.odd_nodes();
        assert_eq!(odd.len(), 5);
        assert!(odd.contains(&NodeId(0)));
        assert!(odd.contains(&NodeId(2)));
    }
}

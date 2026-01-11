use crate::graph::{NodeId, Valences};
use rand::Rng;

/// The 8 symmetries of the square (dihedral group D₄)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symmetry {
    Identity,
    Rot90,
    Rot180,
    Rot270,
    FlipHorizontal,
    FlipVertical,
    FlipMainDiag,
    FlipAntiDiag,
}

impl Symmetry {
    /// Get a random symmetry with uniform distribution
    pub fn random() -> Self {
        let mut rng = rand::rng();
        match rng.random_range(0..8) {
            0 => Symmetry::Identity,
            1 => Symmetry::Rot90,
            2 => Symmetry::Rot180,
            3 => Symmetry::Rot270,
            4 => Symmetry::FlipHorizontal,
            5 => Symmetry::FlipVertical,
            6 => Symmetry::FlipMainDiag,
            _ => Symmetry::FlipAntiDiag,
        }
    }
}

/// Apply a symmetry transformation to valences
pub fn apply_symmetry(valences: &Valences, symmetry: Symmetry) -> Valences {
    let arr = [
        valences.get(NodeId(0)),
        valences.get(NodeId(1)),
        valences.get(NodeId(2)),
        valences.get(NodeId(3)),
        valences.get(NodeId(4)),
        valences.get(NodeId(5)),
        valences.get(NodeId(6)),
        valences.get(NodeId(7)),
        valences.get(NodeId(8)),
    ];

    let transformed = match symmetry {
        Symmetry::Identity => arr,

        // 90° clockwise rotation
        // 0 1 2    6 3 0
        // 3 4 5 -> 7 4 1
        // 6 7 8    8 5 2
        Symmetry::Rot90 => [
            arr[6], arr[3], arr[0], arr[7], arr[4], arr[1], arr[8], arr[5], arr[2],
        ],

        // 180° rotation
        // 0 1 2    8 7 6
        // 3 4 5 -> 5 4 3
        // 6 7 8    2 1 0
        Symmetry::Rot180 => [
            arr[8], arr[7], arr[6], arr[5], arr[4], arr[3], arr[2], arr[1], arr[0],
        ],

        // 270° clockwise (= 90° counter-clockwise)
        // 0 1 2    2 5 8
        // 3 4 5 -> 1 4 7
        // 6 7 8    0 3 6
        Symmetry::Rot270 => [
            arr[2], arr[5], arr[8], arr[1], arr[4], arr[7], arr[0], arr[3], arr[6],
        ],

        // Horizontal flip (left ➡️right)
        // 0 1 2    2 1 0
        // 3 4 5 -> 5 4 3
        // 6 7 8    8 7 6
        Symmetry::FlipHorizontal => [
            arr[2], arr[1], arr[0], arr[5], arr[4], arr[3], arr[8], arr[7], arr[6],
        ],

        // Vertical flip (top ➡️bottom)
        // 0 1 2    6 7 8
        // 3 4 5 -> 3 4 5
        // 6 7 8    0 1 2
        Symmetry::FlipVertical => [
            arr[6], arr[7], arr[8], arr[3], arr[4], arr[5], arr[0], arr[1], arr[2],
        ],

        // Main diagonal transpose (top-left  ➡️bottom-right)
        // 0 1 2    0 3 6
        // 3 4 5 -> 1 4 7
        // 6 7 8    2 5 8
        Symmetry::FlipMainDiag => [
            arr[0], arr[3], arr[6], arr[1], arr[4], arr[7], arr[2], arr[5], arr[8],
        ],

        // Anti-diagonal transpose (top-right ➡️bottom-left)
        // 0 1 2    8 5 2
        // 3 4 5 -> 7 4 1
        // 6 7 8    6 3 0
        Symmetry::FlipAntiDiag => [
            arr[8], arr[5], arr[2], arr[7], arr[4], arr[1], arr[6], arr[3], arr[0],
        ],
    };

    Valences::from_array(transformed)
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Symmetry {
        /// All 8 symmetries in order
        pub fn all() -> [Symmetry; 8] {
            [
                Symmetry::Identity,
                Symmetry::Rot90,
                Symmetry::Rot180,
                Symmetry::Rot270,
                Symmetry::FlipHorizontal,
                Symmetry::FlipVertical,
                Symmetry::FlipMainDiag,
                Symmetry::FlipAntiDiag,
            ]
        }
    }

    #[test]
    fn test_all_symmetries_are_unique() {
        // Apply all 8 symmetries to a non-symmetric puzzle
        let valences = Valences::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);

        let mut results = Vec::new();
        for symmetry in Symmetry::all() {
            let result = apply_symmetry(&valences, symmetry);

            // Convert to vec for easier comparison
            let result_vec: Vec<_> = (0..9).map(|i| result.get(NodeId(i))).collect();

            // Check this result is unique
            assert!(
                !results.contains(&result_vec),
                "Symmetry {:?} produced duplicate result",
                symmetry
            );

            results.push(result_vec);
        }

        // Should have exactly 8 unique results
        assert_eq!(results.len(), 8);
    }

    #[test]
    fn test_identity() {
        let valences = Valences::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let result = apply_symmetry(&valences, Symmetry::Identity);

        for i in 0..9 {
            assert_eq!(result.get(NodeId(i)), valences.get(NodeId(i)));
        }
    }

    #[test]
    fn test_symmetric_puzzle_has_fewer_unique_transforms() {
        // A puzzle with 4-fold rotational symmetry
        let symmetric = Valences::new(vec![1, 2, 1, 2, 5, 2, 1, 2, 1]);

        let mut unique_results = std::collections::HashSet::new();
        for symmetry in Symmetry::all() {
            let result = apply_symmetry(&symmetric, symmetry);
            let result_vec: Vec<_> = (0..9).map(|i| result.get(NodeId(i))).collect();
            unique_results.insert(result_vec);
        }

        // This symmetric puzzle should have fewer than 8 unique results
        assert!(unique_results.len() < 8);
    }

    #[test]
    fn test_rot90_composition() {
        // Applying Rot90 four times should give identity
        let valences = Valences::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);

        let mut result = valences.clone();
        for _ in 0..4 {
            result = apply_symmetry(&result, Symmetry::Rot90);
        }

        for i in 0..9 {
            assert_eq!(result.get(NodeId(i)), valences.get(NodeId(i)));
        }
    }
}

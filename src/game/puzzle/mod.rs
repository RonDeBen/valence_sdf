mod transforms;

use crate::graph::Valences;
use bevy::prelude::*;
use rand::prelude::*;
use rand::rng;
use std::collections::HashMap;
pub use transforms::{Symmetry, apply_symmetry};

const PUZZLES_CSV: &str = include_str!("../../../assets/puzzles_symmetric.csv");

/// Resource containing all base puzzles organized by complexity
#[derive(Resource, Debug)]
pub struct PuzzleLibrary {
    puzzles_by_complexity: HashMap<usize, Vec<BasePuzzle>>,
}

/// A base puzzle before geometric transformations are applied
#[derive(Debug, Clone)]
struct BasePuzzle {
    valences: Valences,
}

/// Configuration for a single puzzle instance
#[derive(Debug, Clone)]
pub struct PuzzleConfig {
    pub valences: Valences,
    pub complexity: usize,
    pub total_solutions: usize,
}

impl PuzzleLibrary {
    /// Load the puzzle library from embedded CSV data
    pub fn load() -> Result<Self, String> {
        Self::from_csv(PUZZLES_CSV)
    }

    /// Parse CSV data into puzzle library
    ///
    /// CSV format: 9 valence values followed by complexity
    /// Example: 0,0,0,0,0,0,0,1,1,1
    fn from_csv(csv_data: &str) -> Result<Self, String> {
        let mut puzzles_by_complexity: HashMap<usize, Vec<BasePuzzle>> = HashMap::new();

        for (line_num, line) in csv_data.lines().enumerate() {
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            let values: Result<Vec<usize>, _> =
                line.split(',').map(|s| s.trim().parse::<usize>()).collect();

            let values =
                values.map_err(|e| format!("Parse error on line {}: {}", line_num + 1, e))?;

            if values.len() != 10 {
                return Err(format!(
                    "Line {} has {} values, expected 10 (9 valences + 1 complexity)",
                    line_num + 1,
                    values.len()
                ));
            }

            let complexity = values[9];
            let valences = Valences::new(values[0..9].to_vec());

            puzzles_by_complexity
                .entry(complexity)
                .or_default()
                .push(BasePuzzle { valences });
        }

        if puzzles_by_complexity.is_empty() {
            return Err("No puzzles loaded from CSV".to_string());
        }

        Ok(PuzzleLibrary {
            puzzles_by_complexity,
        })
    }

    /// Get a random puzzle of given complexity with random geometric transform
    pub fn random_puzzle(&self, complexity: usize) -> Option<PuzzleConfig> {
        let base_puzzles = self.puzzles_by_complexity.get(&complexity)?;
        let base = base_puzzles.choose(&mut rng())?;

        // Apply random symmetric transform
        let transform = Symmetry::random();
        let valences = apply_symmetry(&base.valences, transform);
        let total_solutions = self.solution_count_for_puzzle(&valences, complexity);

        Some(PuzzleConfig {
            valences,
            complexity,
            total_solutions,
        })
    }

    /// Get a specific untried puzzle (for level tour mode)
    ///
    /// Returns the puzzle config and the base puzzle index so it can be tracked
    pub fn untried_puzzle(
        &self,
        complexity: usize,
        tried_indices: &[usize],
    ) -> Option<(PuzzleConfig, usize)> {
        let base_puzzles = self.puzzles_by_complexity.get(&complexity)?;

        // Find all untried puzzles
        let untried: Vec<_> = base_puzzles
            .iter()
            .enumerate()
            .filter(|(idx, _)| !tried_indices.contains(idx))
            .collect();

        if untried.is_empty() {
            return None;
        }

        // Pick a random untried puzzle
        let (puzzle_idx, base) = untried.choose(&mut rng())?;

        // Apply random transform
        let transform = Symmetry::random();
        let valences = apply_symmetry(&base.valences, transform);
        let total_solutions = self.solution_count_for_puzzle(&valences, complexity);

        let config = PuzzleConfig {
            valences,
            complexity,
            total_solutions,
        };

        Some((config, *puzzle_idx))
    }

    /// Get the number of base puzzles for a given complexity
    pub fn puzzle_count(&self, complexity: usize) -> usize {
        self.puzzles_by_complexity
            .get(&complexity)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Get all available complexity levels, sorted
    pub fn available_complexities(&self) -> Vec<usize> {
        let mut complexities: Vec<_> = self.puzzles_by_complexity.keys().copied().collect();
        complexities.sort_unstable();
        complexities
    }

    /// Get the total number of base puzzles across all complexities
    pub fn total_puzzle_count(&self) -> usize {
        self.puzzles_by_complexity.values().map(|v| v.len()).sum()
    }

    fn solution_count_for_puzzle(&self, valences: &Valences, complexity: usize) -> usize {
        let num_edges = valences.total() / 2;
        complexity / num_edges
    }
}

/// System to load and initialize the puzzle library
/// This should run early in Startup schedule, before setup_puzzle
pub fn setup_puzzle_library(mut commands: Commands) {
    match PuzzleLibrary::load() {
        Ok(library) => {
            let complexities = library.available_complexities();
            let total_puzzles = library.total_puzzle_count();

            info!("✓ Puzzle library loaded successfully:");
            info!("  - {} unique complexity levels", complexities.len());
            info!("  - {} total base puzzles", total_puzzles);
            info!(
                "  - Complexity range: {} to {}",
                complexities.first().unwrap_or(&0),
                complexities.last().unwrap_or(&0)
            );

            // Log some details about puzzle distribution
            for &complexity in complexities.iter().take(5) {
                let count = library.puzzle_count(complexity);
                info!("  - Complexity {}: {} base puzzles", complexity, count);
            }
            if complexities.len() > 5 {
                info!(
                    "  - ... and {} more complexity levels",
                    complexities.len() - 5
                );
            }

            commands.insert_resource(library);
        }
        Err(e) => {
            error!("Failed to load puzzle library: {}", e);
            panic!("Cannot continue without puzzle data");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CSV: &str = "\
0,0,0,0,0,0,0,1,1,1
0,0,0,0,0,1,0,1,0,1
0,0,0,0,1,0,0,0,1,1
0,0,0,0,0,0,1,2,1,2
0,0,0,0,0,1,0,1,2,2";

    #[test]
    fn test_load_from_csv() {
        let library = PuzzleLibrary::from_csv(TEST_CSV).unwrap();

        assert_eq!(library.puzzle_count(1), 3);
        assert_eq!(library.puzzle_count(2), 2);
        assert_eq!(library.total_puzzle_count(), 5);
    }

    #[test]
    fn test_available_complexities() {
        let library = PuzzleLibrary::from_csv(TEST_CSV).unwrap();
        let complexities = library.available_complexities();

        assert_eq!(complexities, vec![1, 2]);
    }

    #[test]
    fn test_random_puzzle() {
        let library = PuzzleLibrary::from_csv(TEST_CSV).unwrap();

        // Should be able to get puzzles for complexity 1 and 2
        assert!(library.random_puzzle(1).is_some());
        assert!(library.random_puzzle(2).is_some());

        // Should return None for non-existent complexity
        assert!(library.random_puzzle(999).is_none());
    }

    #[test]
    fn test_untried_puzzle() {
        let library = PuzzleLibrary::from_csv(TEST_CSV).unwrap();

        // First call should succeed
        let (config1, idx1) = library.untried_puzzle(1, &[]).unwrap();
        assert_eq!(config1.complexity, 1);

        // Can get another one
        let (_, idx2) = library.untried_puzzle(1, &[idx1]).unwrap();
        assert_ne!(idx1, idx2);

        // Can get a third
        let (_, idx3) = library.untried_puzzle(1, &[idx1, idx2]).unwrap();
        assert_ne!(idx3, idx1);
        assert_ne!(idx3, idx2);

        // After marking all 3 as tried, should return None
        assert!(library.untried_puzzle(1, &[idx1, idx2, idx3]).is_none());
    }

    #[test]
    fn test_invalid_csv() {
        // Too few values
        let bad_csv = "0,0,0,0,0,1";
        assert!(PuzzleLibrary::from_csv(bad_csv).is_err());

        // Non-numeric values
        let bad_csv2 = "0,0,0,x,0,0,0,1,1,1";
        assert!(PuzzleLibrary::from_csv(bad_csv2).is_err());
    }

    #[test]
    fn test_empty_csv() {
        assert!(PuzzleLibrary::from_csv("").is_err());
    }
}

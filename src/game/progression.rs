// game/progression.rs

use bevy::prelude::*;

/// Maps level number (1-217) to complexity value
/// Generated from the unique complexity values in the symmetric puzzles CSV
const LEVEL_TO_COMPLEXITY: &[usize] = &[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 20, 21,
    22, 24, 25, 26, 27, 28, 30, 32, 33, 34, 35, 36, 39, 40, 42, 44, 45, 48,
    49, 50, 52, 54, 55, 56, 60, 63, 64, 65, 66, 70, 72, 75, 77, 78, 80, 81,
    84, 88, 90, 91, 96, 98, 99, 100, 104, 105, 108, 110, 112, 117, 120, 121,
    126, 128, 130, 132, 135, 136, 140, 143, 144, 147, 150, 152, 153, 154, 156,
    160, 161, 162, 165, 168, 169, 170, 171, 175, 176, 180, 182, 184, 187, 189,
    190, 192, 195, 196, 198, 200, 204, 207, 208, 209, 210, 216, 220, 221, 224,
    225, 228, 230, 231, 232, 234, 240, 242, 248, 250, 252, 253, 260, 261, 264,
    270, 279, 280, 285, 286, 288, 294, 297, 299, 300, 304, 306, 308, 310, 312,
    319, 320, 322, 325, 330, 333, 336, 338, 342, 348, 350, 351, 352, 360, 363,
    364, 368, 370, 372, 374, 376, 378, 384, 390, 392, 418, 420, 423, 429, 430,
    432, 440, 450, 470, 494, 500, 504, 517, 532, 533, 540, 550, 570, 576, 583,
    594, 600, 624, 630, 650, 663, 671, 672, 676, 684, 696, 700, 708, 728, 732,
    740, 792, 810, 832, 852, 858, 880, 924, 936, 960,
];

const MAX_LEVEL: usize = 217;

/// Resource tracking progression through the 217 complexity levels
#[derive(Resource, Debug)]
pub struct ProgressionTracker {
    /// Current level (1-217)
    pub current_level: usize,
    /// Puzzles completed at current level
    pub completed_at_level: usize,
}

impl Default for ProgressionTracker {
    fn default() -> Self {
        Self {
            current_level: 1,
            completed_at_level: 0,
        }
    }
}

impl ProgressionTracker {
    /// Get the complexity value for the current level
    pub fn current_complexity(&self) -> usize {
        LEVEL_TO_COMPLEXITY[self.current_level - 1]
    }
    
    /// Advance to next level, wrapping around if at end
    pub fn advance_level(&mut self) {
        self.current_level = if self.current_level >= MAX_LEVEL {
            1
        } else {
            self.current_level + 1
        };
        self.completed_at_level = 0;
    }
    
    /// Get progress as a percentage (0.0 to 100.0)
    pub fn progress_percentage(&self) -> f32 {
        (self.current_level as f32 / MAX_LEVEL as f32) * 100.0
    }
    
    /// Check if this is the final level
    pub fn is_final_level(&self) -> bool {
        self.current_level == MAX_LEVEL
    }
    
    /// Get the total number of levels
    pub fn max_level() -> usize {
        MAX_LEVEL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_to_complexity_has_217_entries() {
        assert_eq!(LEVEL_TO_COMPLEXITY.len(), 217);
    }

    #[test]
    fn test_level_to_complexity_is_sorted() {
        for i in 1..LEVEL_TO_COMPLEXITY.len() {
            assert!(
                LEVEL_TO_COMPLEXITY[i] >= LEVEL_TO_COMPLEXITY[i - 1],
                "Complexity values should be non-decreasing"
            );
        }
    }

    #[test]
    fn test_tracker_default() {
        let tracker = ProgressionTracker::default();
        assert_eq!(tracker.current_level, 1);
        assert_eq!(tracker.current_complexity(), 1);
    }

    #[test]
    fn test_advance_level() {
        let mut tracker = ProgressionTracker::default();
        
        assert_eq!(tracker.current_level, 1);
        tracker.advance_level();
        assert_eq!(tracker.current_level, 2);
        assert_eq!(tracker.current_complexity(), 2);
    }

    #[test]
    fn test_advance_level_wraps() {
        let mut tracker = ProgressionTracker {
            current_level: 217,
            completed_at_level: 0,
        };
        
        tracker.advance_level();
        assert_eq!(tracker.current_level, 1);
        assert_eq!(tracker.current_complexity(), 1);
    }

    #[test]
    fn test_progress_percentage() {
        let tracker = ProgressionTracker {
            current_level: 109,
            completed_at_level: 0,
        };
        
        let percentage = tracker.progress_percentage();
        assert!((percentage - 50.23).abs() < 0.1); // Approximately 50%
    }

    #[test]
    fn test_is_final_level() {
        let mut tracker = ProgressionTracker {
            current_level: 216,
            completed_at_level: 0,
        };
        
        assert!(!tracker.is_final_level());
        
        tracker.advance_level();
        assert!(tracker.is_final_level());
    }

    #[test]
    fn test_known_complexity_values() {
        // Test some known level-to-complexity mappings
        let mut tracker = ProgressionTracker::default();
        
        // Level 1 should be complexity 1
        tracker.current_level = 1;
        assert_eq!(tracker.current_complexity(), 1);
        
        // Level 19 should be complexity 20 (gap at 19)
        tracker.current_level = 19;
        assert_eq!(tracker.current_complexity(), 20);
        
        // Level 217 should be complexity 960 (highest)
        tracker.current_level = 217;
        assert_eq!(tracker.current_complexity(), 960);
    }
}

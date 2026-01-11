use bevy::prelude::*;

use crate::game::{
    progression::ProgressionTracker,
    puzzle::PuzzleLibrary,
    session::PuzzleSession,
};

/// System: Setup the puzzle session from the library
/// This runs after setup_puzzle_library, which loads the CSV data
pub fn setup_puzzle(mut commands: Commands, library: Res<PuzzleLibrary>) {
    let tracker = ProgressionTracker::default();
    let complexity = tracker.current_complexity();

    let config = library
        .random_puzzle(complexity)
        .expect("No puzzles available for starting level");

    info!(
        "🎮 Level {}: complexity {}, {} solutions expected",
        tracker.current_level,
        config.complexity,
        config.total_solutions
    );

    let session = PuzzleSession::new(config.valences, config.total_solutions);

    commands.insert_resource(tracker);
    commands.insert_resource(session);
}

/// System: Check for level completion and advance to next level
/// This should run in the Update schedule
pub fn check_level_progression(
    mut commands: Commands,
    session: Res<PuzzleSession>,
    mut tracker: ResMut<ProgressionTracker>,
    library: Res<PuzzleLibrary>,
) {
    // Only check when the session has changed (e.g., new solution found)
    if !session.is_changed() {
        return;
    }

    // Check if ALL solutions have been found for this puzzle
    let progress = session.progress();
    if !progress.is_complete() {
        return;
    }

    info!("🎉 Level {} complete! All solutions found!", tracker.current_level);

    tracker.advance_level();
    let complexity = tracker.current_complexity();

    if tracker.current_level == 1 {
        info!("🏆 You've completed all 217 levels! Starting over...");
    }

    if let Some(config) = library.random_puzzle(complexity) {
        info!(
            "🎮 Level {}/{}: complexity {}, {} solutions expected",
            tracker.current_level,
            ProgressionTracker::max_level(),
            config.complexity,
            config.total_solutions
        );

        let new_session = PuzzleSession::new(config.valences, config.total_solutions);
        commands.insert_resource(new_session);
    } else {
        error!(
            "❌ No puzzle found for level {} (complexity {})",
            tracker.current_level, complexity
        );
    }
}


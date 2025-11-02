use bevy::prelude::*;

use crate::{
    game::session::PuzzleSession,
    graph::Valences,
};

/// System: Setup the puzzle session
pub fn setup_puzzle(mut commands: Commands) {
    // hardcoded puzzle for now
    let valences = Valences::new(vec![2, 4, 2, 4, 8, 4, 2, 5, 3]);
    let session = PuzzleSession::new(valences, 1);

    commands.insert_resource(session);
}


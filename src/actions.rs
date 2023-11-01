//

use bevy::prelude::{Plugin, PreStartup, PreUpdate, App};

mod move_actions;
mod node_actions;
mod edge_actions;

// All undoable actions must implement this trait
pub trait Action {
    fn execute(&self);
    fn undo(&self);
}

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreStartup, move_actions::setup_move_actions)
        ;
    }
}
use bevy::prelude::{Plugin, App, Resource}; // import InitResource trait

use crate::actions::Action;

#[derive(Resource)]
struct ActionHistory {
    actions: Vec<String>,
}

impl ActionHistory {
    fn add_action(&mut self, action: String) {
        self.actions.push(action);
    }
}

impl Default for ActionHistory {
    fn default() -> Self {
        ActionHistory {
            actions: Vec::new(),
        }
    }
}

pub struct HistoryPlugin;

impl Plugin for HistoryPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ActionHistory::default());
    }
}
use bevy::prelude::{Plugin, App};

use self::background::RectangleSelectionEvent;
use self::context::*;
use self::nodes::*;
use self::edges::*;

pub mod background;
pub mod context;
pub mod nodes;
pub mod edges;

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app
            // Background events
            .add_event::<RectangleSelectionEvent>()
            
            // Node Events
            .add_event::<NodeSpawnedEvent>()
            .add_event::<NodeClickEvent>()
            .add_event::<NodePressedEvent>()
            .add_event::<NodeHoverEvent>()
            .add_event::<MoveNodesEvent>()

            // Edge Events
            .add_event::<EdgeSpawnedEvent>()
        ;
    }
}
use bevy::prelude::{Plugin, App};

use self::background::*;
use self::node_events::*;
use self::edges::*;

pub mod background;
pub mod node_events;
pub mod edges;

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app
            // Background events
            .add_event::<RectangleSelectionEvent>()
            .add_event::<RectangleSelectionEndEvent>()

            // Node Events
            .add_event::<NodeClickEvent>()
            .add_event::<NodePressedEvent>()
            .add_event::<NodeHoverEvent>()
            .add_event::<NodeHoverStopEvent>()
            .add_event::<MoveNodesEvent>()

            // Edge Events
            .add_event::<EdgeClickEvent>()
        ;
    }
}
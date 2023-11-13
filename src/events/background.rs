// Events that are triggered upon interactions with the graph background

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

#[derive(Event)]
pub struct RectangleSelectionEvent {
    pub start: Option<Vec3>,
}

// Implementation required by bevy_mod_picking
impl From<ListenerInput<Pointer<Down>>> for RectangleSelectionEvent {
    fn from(event: ListenerInput<Pointer<Down>>) -> Self {
        RectangleSelectionEvent {
            start: event.hit.position,
        }
    }
}

#[derive(Event)]
pub struct RectangleSelectionEndEvent {
    pub end: Vec2,
}

// Implementation required by bevy_mod_picking
impl From<ListenerInput<Pointer<DragEnd>>> for RectangleSelectionEndEvent {
    fn from(event: ListenerInput<Pointer<DragEnd>>) -> Self {
        RectangleSelectionEndEvent {
            end: event.distance,
        }
    }
}
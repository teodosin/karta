//


use bevy::prelude::{Event, Entity};
use bevy_mod_picking::prelude::*;


#[derive(Event)]   
pub struct MoveNodesEvent;

// Implementation required by bevy_mod_picking
impl From<ListenerInput<Pointer<Drag>>> for MoveNodesEvent {
    fn from(_event: ListenerInput<Pointer<Drag>>) -> Self {
        MoveNodesEvent
    }
}

// The main input event for nodes. 
#[derive(Event)]
pub struct NodeClickEvent {
    pub target: Option<Entity>,
    pub button: PointerButton,
}

// Implementation required by bevy_mod_picking
impl From<ListenerInput<Pointer<Click>>> for NodeClickEvent {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        NodeClickEvent {
            target: Some(event.target),
            button: event.button
        }
    }
}

#[derive(Event)]
pub struct NodePressedEvent {
    pub target: Option<Entity>,
    pub button: PointerButton,

}

impl From<ListenerInput<Pointer<Down>>> for NodePressedEvent {
    fn from(event: ListenerInput<Pointer<Down>>) -> Self {
        NodePressedEvent {
            target: Some(event.target),
            button: event.button
        }
    }
}

#[derive(Event)]
pub struct NodeHoverEvent {
    pub target: Option<Entity>,
}

// Implementation required by bevy_mod_picking
impl From<ListenerInput<Pointer<Over>>> for NodeHoverEvent {
    fn from(event: ListenerInput<Pointer<Over>>) -> Self {
        NodeHoverEvent {
            target: Some(event.target),
        }
    }
}

#[derive(Event)]
pub struct NodeHoverStopEvent;

// Implementation required by bevy_mod_picking
impl From<ListenerInput<Pointer<Out>>> for NodeHoverStopEvent {
    fn from(_event: ListenerInput<Pointer<Out>>) -> Self {
        NodeHoverStopEvent
    }
}
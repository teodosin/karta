//

use bevy::prelude::{Event, Entity, MouseButton};
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
    pub button: MouseButton,
}

// Implementation required by bevy_mod_picking
impl From<ListenerInput<Pointer<Click>>> for NodeClickEvent {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        NodeClickEvent {
            target: Some(event.target),
            button: match event.button {
                // It's interesting that bevy mod picking and bevy input 
                // use different enums for mouse buttons. Asked the crate
                // author on Discord about it. 
                PointerButton::Primary => MouseButton::Left,
                PointerButton::Secondary => MouseButton::Right,
                PointerButton::Middle => MouseButton::Middle,
                //_ => MouseButton::Other,
            }
        }
    }
}

#[derive(Event)]
pub struct NodePressedEvent {
    pub target: Option<Entity>,
    pub button: MouseButton,

}

impl From<ListenerInput<Pointer<Down>>> for NodePressedEvent {
    fn from(event: ListenerInput<Pointer<Down>>) -> Self {
        NodePressedEvent {
            target: Some(event.target),
            button: match event.button {
                PointerButton::Primary => MouseButton::Left,
                PointerButton::Secondary => MouseButton::Right,
                PointerButton::Middle => MouseButton::Middle,
            }
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
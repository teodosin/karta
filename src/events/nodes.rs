//

use bevy::prelude::{Event, Entity, MouseButton, Vec2};
use bevy_mod_picking::prelude::*;

use crate::graph::node_types::NodeTypes;


// Informing the World that a node has been spawned
// Mostly needed to add ui to the node, which before this is just the 
// bare data.
#[derive(Event)]
pub struct NodeSpawnedEvent {
    pub node: Entity,
    pub path: String,
    pub name: String,
    pub ntype: NodeTypes,
    pub position: Vec2,
}

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
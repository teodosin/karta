//

use bevy::prelude::{Entity, With, Vec2, EventWriter, Commands};

use crate::{graph::{nodes::{PinnedToPosition, spawn_node, GraphDataNode}, context::{PathsToEntitiesIndex, Selected, CurrentContext}, node_types::NodeTypes}, input::pointer::InputData, ui::nodes::GraphViewNode, events::nodes::NodeSpawnedEvent};

use super::Action;

// ------------------ CreateNodeAction ------------------

#[derive(Clone)]
pub struct CreateNodeAction {
    // entity: Entity,
    ntype: NodeTypes,
    position: Vec2,
}

impl Action for CreateNodeAction {
    // NOTE: The implementation here must be kept in sync with the implementation of spawn_node,
    // which is mostly called when expanding a node or changing context. 
    fn execute(&mut self, world: &mut bevy::prelude::World) {
        println!("Creating: ");

        let context = world.get_resource::<CurrentContext>().unwrap();

        // TODO: Implement a function to get an available name. Use the Houdini convention.
        // Two nodes can't share the same path, so we need to check if a node already exists
        // in the current path. 
        let name = self.ntype.to_string();
        let path = context.get_current_context_path();
        let full_path = format!("{}/{}", path, name);

        let node_entity = world.spawn((
            GraphDataNode {
                path: full_path.clone(),
                name: name.clone()
            },
            PinnedToPosition,
        )).id();

        world.send_event(NodeSpawnedEvent {
            entity: node_entity,
            path: path.to_string(),
            name: name.to_string(),
            ntype: self.ntype,
            position: self.position,
        });
    
        // Update the PathsToEntitiesIndex
        let mut pe_index = world.get_resource_mut::<PathsToEntitiesIndex>().unwrap();
        pe_index.0.insert(full_path, node_entity);
    }

    fn undo(&mut self, world: &mut bevy::prelude::World) {
        println!("Undoing CreateNodeAction");
    }

    fn redo(&mut self, world:  &mut bevy::prelude::World) {
        println!("Redoing CreateNodeAction");
    }
}

// impl Default for CreateNodeAction {
//     fn default() -> Self {
//         CreateNodeAction {
//             node: String::from("Default Node"),
//         }
//     }
// }

impl CreateNodeAction {
    pub fn new(
        // entity: Entity,
        ntype: NodeTypes, 
        pos: Vec2
    ) -> Self {
        CreateNodeAction {
            // entity,
            ntype, 
            position: pos,
        }
    }
}

// ------------------ DeleteNodeAction ------------------

// ------------------ RemoveNodeAction ------------------
// There is a distinction to be made between removing and deleting. Removing a node
// just removes it from the current context. If the node doesn't exist in any other context,
// it will also be deleted after a warning. If the current context is the parent context,
// but it exists in other contexts, upon removal the user should also probably be 
// notified if the node has to be moved to a different host context.

// ------------------ EditNodeAction ------------------

// ------------------ PinToPositionAction ------------------
#[derive(Clone)]
pub struct PinToPositionAction {
    pins: Option<Vec<(Entity, bool)>>,
}

impl Action for PinToPositionAction {
    fn execute(&mut self, world: &mut bevy::prelude::World) {
        let mut selected = world.query_filtered::<Entity, (With<GraphViewNode>, With<Selected>)>();
        let mut pinned = world.query_filtered::<Entity, (With<GraphViewNode>, With<PinnedToPosition>)>();

        let mut targets: Vec<Entity> = Vec::new();
        
        for node in selected.iter(world) {
            println!("Looping through selected");
            match pinned.get(world, node){
                Ok(_) => {
                    println!("It's a match in pinven");
                    match self.pins {
                        Some(ref mut pins) => pins.push((node, true)),
                        None => self.pins = Some(vec![(node, true)]),
                    }
                }
                Err(_) => {
                    targets.push(node);
                    match self.pins {
                        Some(ref mut pins) => pins.push((node, false)),
                        None => self.pins = Some(vec![(node, false)]),
                    }
                }
            }
        }
        for target in targets {
            println!("Adding pin component");
            world.entity_mut(target).insert(PinnedToPosition);
        }
        
        println!("Performing PinToPositionAction");
    }

    fn undo(&mut self, world: &mut bevy::prelude::World) {
        let node = self.get_latest_clicked_node(world).unwrap();
        world.entity_mut(node).remove::<PinnedToPosition>();
        println!("Undoing PinToPositionAction");
    }

    fn redo(&mut self, world:  &mut bevy::prelude::World) {
        let node = self.get_latest_clicked_node(world).unwrap();
        world.entity_mut(node).insert(PinnedToPosition);
        println!("Redoing PinToPositionAction");
    }
}

impl PinToPositionAction {
    pub fn new() -> Self {
        PinToPositionAction {
            pins: None,
        }
    }

    fn get_latest_clicked_node(
        &self, world: &mut bevy::prelude::World
    ) -> Option<Entity> {
        let input_data = world.get_resource::<InputData>().unwrap();
        let path = input_data.latest_click_entity.clone().unwrap();
        let index = world.get_resource::<PathsToEntitiesIndex>().unwrap();
        let node = index.0.get(&path);
        Some(*node.unwrap())
    }
}

// ------------------ UnpinToPositionAction ------------------
#[derive(Clone)]
pub struct UnpinToPositionAction {
    node: Entity,
}

impl Action for UnpinToPositionAction {
    fn execute(&mut self, world: &mut bevy::prelude::World) {
        world.entity_mut(self.node).remove::<PinnedToPosition>();
        println!("Performing UnpinToPositionAction");
    }

    fn undo(&mut self, world: &mut bevy::prelude::World) {
        world.entity_mut(self.node).insert(PinnedToPosition);
        println!("Undoing UnpinToPositionAction");
    }

    fn redo(&mut self, world:  &mut bevy::prelude::World) {
        world.entity_mut(self.node).remove::<PinnedToPosition>();
        println!("Redoing UnpinToPositionAction");
    }

}

impl UnpinToPositionAction {
    pub fn new(node: Entity) -> Self {
        UnpinToPositionAction {
            node,
        }
    }
}


// ------------------ PinToPresenceAction ------------------

// ------------------ UnpinToPresenceAction ------------------